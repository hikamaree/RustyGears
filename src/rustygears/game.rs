// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of Rusty Gears.
//
// Rusty Gears is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rusty Gears is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::GearMessage;
use crate::Logs;
use crate::ComponentMap;
use crate::Input;
use crate::Model3d;
use crate::Command;
use crate::BindGroupLayoutKey;
use crate::RenderObject;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Time;
use crate::GameView;
use crate::WorldScene;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;

use winit::event_loop::EventLoop;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use tokio::runtime::Runtime;

/// Represents the core application state, managing rendering, scene data, time progression,
/// and communication with worker threads ("gears").
///
/// The `Game` struct acts as the central access point for subsystems such as the ECS-based world scene,
/// graphics backend, and rendering pipelines. It also manages runtime model loading and entity spawning.
///
/// # Fields
/// - `setupfns`: Queue of initialization functions executed during game setup.
/// - `gear_channels`: Channels used to communicate with asynchronous worker systems ("gears").
/// - `command_receiver`: Channel for receiving commands from the game engine to be executed.
/// - `runtime`: Tokio runtime for executing asynchronous tasks on background threads.
/// - `components`: Central component registry used to store and access engine-wide subsystems and shared state via type-safe accessors.
pub struct Game {
    pub(crate) setupfns: VecDeque<Box<dyn FnOnce(&mut Game) + Send>>,
    pub(crate) gear_channels: HashMap<String, Sender<GearMessage>>,
    pub(crate) command_receiver: Receiver<Box<dyn Command>>,
    pub runtime: Runtime,
    pub components: ComponentMap,
}

impl Game {
    /// Creates a new `Game` instance with default components.
    ///
    /// # Returns
    /// A new `Game` instance.
    pub fn new() -> Self {
        let (command_sender, command_receiver) = crossbeam::channel::unbounded();
        crate::init_command_sender(command_sender);

        let runtime = tokio::runtime::Runtime::new().expect("ERROR: Failed to create tokio runtime");

        let mut components = ComponentMap::new();

        components.insert(WorldScene::default());
        components.insert(Time::new());
        components.insert(Input::new());
        components.insert(Logs::default());

        Self {
            setupfns: VecDeque::new(),
            gear_channels: HashMap::new(),
            command_receiver,
            runtime,
            components,
        }
    }

    /// Starts the main game loop using the event system.
    ///
    /// Creates a new `EventLoop` and begins running the application,
    /// passing a mutable reference to `self`.
    ///
    /// # Panics
    /// If the event loop fails to initialize or run.
    pub fn run(mut self) {
        let game_loop = EventLoop::new().expect("ERROR: Failed to crate game loop");
        game_loop.run_app(&mut self).expect("ERROR: Failed to run game loop");
    }

    /// Adds a new gear to the game and starts its processing thread.
    ///
    /// The gear is initialized via its `setup` method, and then run in a separate thread
    /// where it listens for `GearMessage` messages. These events are dispatched from the main
    /// game loop and routed to the appropriate gear methods (`update`, `mouse_motion`, etc.).
    ///
    /// The gear is registered under the provided `id`, and any previously existing gear with the same
    /// identifier will be silently replaced.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for the gear.
    /// * `gear` - An instance of a type that implements the `Gear` trait.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.
    pub fn add_gear<T: Gear + 'static>(&mut self, id: String, mut gear: T) -> &mut Self {
        gear.setup(self);
        let (gear_sender, gear_receiver) = crossbeam::channel::unbounded();
        self.gear_channels.insert(id.clone(), gear_sender.clone());

        std::thread::spawn(move || {
            while let Ok(msg) = gear_receiver.recv() {
                match msg.gear_event {
                    GearEvent::Update() => {
                        gear.update(msg.game);
                        crate::send_command(super::UpdateDoneCommand);
                    },
                    GearEvent::Exit() => {
                        gear.exit(msg.game);
                        crate::send_command(super::UpdateDoneCommand);
                        break;
                    }
                }
            }
        });

        self
    }

    /// Queues a setup function to be called later during initialization.
    ///
    /// # Arguments
    /// * `setupfn` - A closure that takes a mutable reference to the game instance
    ///   and performs any desired setup logic (e.g. spawning models, adding systems).
    ///
    /// The setup functions are deferred and can be executed later in a controlled manner.
    ///
    /// # Returns
    /// A new `Game` instance with the setup function added to its queue.
    pub fn setup<F: FnOnce(&mut Game) + Send + 'static>(mut self, setupfn: F) -> Self { 
        self.setupfns.push_back(Box::new(setupfn));
        self
    }

    /// Dispatches a [`GearEvent`] to all active gears registered in the game.
    ///
    /// For each gear, a [`GearMessage`] is constructed, bundling the provided event and
    /// a snapshot of the current game state via a [`GameView`] reference. This snapshot includes
    /// access to `Graphics`, `Time`, and `Scene`, allowing each gear to process the event
    /// with full game context.
    ///
    /// The event is sent through a dedicated channel (`Sender<GearMessage>`) for each gear.
    /// Each gear runs in its own thread and receives events asynchronously. If a gear's receiving
    /// thread has exited or panicked (i.e., the channel is disconnected), the gear is considered
    /// dead and is removed from both `gear_channels` and `gear_handles`.
    ///
    /// After all events are dispatched, any pending [`Command`]s returned by gears (sent via
    /// a shared command channel) are drained and applied immediately to the game state.
    ///
    /// # Arguments
    ///
    /// * `event` - The [`GearEvent`] to broadcast to all active gears.
    ///
    /// # Behavior
    ///
    /// - Dead gears (i.e., those whose channel has been disconnected) are silently removed.
    /// - No panic occurs if sending to a gear fails.
    /// - Commands sent from gears during this cycle are executed immediately after dispatching.
    ///
    /// # Example
    ///
    /// ```rust
    /// game.dispatch_event(GearEvent::Update());
    /// ```
    pub fn dispatch_event(&mut self, event: GearEvent) {
        let engine = self.components.get_view();
        
        let mut dead_gears = Vec::new();

        for (id, sender) in &self.gear_channels {
            let gear_event = event.clone();

            let result = sender.send(GearMessage {
                gear_event,
                game: GameView {
                    components: engine.clone(),
                },
            });

            if let Err(_err) = result {
                dead_gears.push(id.clone());
            }
        }

        for id in dead_gears {
            self.gear_channels.remove(&id);
        }

        let mut pending_gear_updates = self.gear_channels.len();

        let mut commands: Vec<Box<dyn Command>> = Vec::new();

        while pending_gear_updates > 0 {
            if let Ok(cmd) = self.command_receiver.recv() {
                if (&*cmd as &dyn std::any::Any).downcast_ref::<crate::UpdateDoneCommand>().is_some() {
                    pending_gear_updates -= 1;
                } else {
                    commands.push(cmd);
                }
            }
        }

        for cmd in commands {
            cmd.apply(self);
        }
    }

    /// Loads a model and registers it in the scene's render object registry.
    ///
    /// This function searches for `.obj` files with LOD variants that share the same
    /// filename prefix. All found LODs are loaded into GPU memory, and stored in a
    /// [`RenderObject`] under the provided name key.
    ///
    /// # Parameters
    /// - `name`: Model key and relative path under the `resources/` directory
    ///           (e.g., `"tree/tree.obj"`).
    ///
    /// # Returns
    /// - `Ok(())` if the model and all its LODs are successfully loaded and registered.
    /// - `Err(String)` if an error occurs during path resolution, file reading, or GPU upload.
    ///
    /// # Errors
    /// This function returns an error if:
    /// - The model file path is invalid or contains non-UTF8 characters.
    /// - The containing directory can't be read.
    /// - No matching `.obj` LOD files are found.
    /// - Loading any of the model LODs fails.
    /// - Required GPU resources (e.g., bind group layouts) are missing.
    pub fn load_model(&mut self, name: &str) -> Result<Model3d, String> {
        let scene = match self.components.get_mut::<WorldScene>() {
            Ok(scene) => scene,
            Err(e) => return Err(e),
        };

        let graphics = match self.components.get_mut::<Graphics>() {
            Ok(graphics) => graphics,
            Err(e) => return Err(e),
        };


        let model = Model3d {
            path: name.to_string()
        };

        if scene.render_objects.contains_key(&model) {
            return Ok(model);
        }

        let Some(texture_layout) = graphics.bind_group_layouts.get(&BindGroupLayoutKey::Texture) else {
            return Err("Missing texture bind group layout".into());
        };

        let base_path = Path::new("resources").join(name);
        let Some(parent_dir) = base_path.parent() else {
            return Err(format!("Invalid model path: {}", base_path.display()));
        };

        let Some(stem_osstr) = base_path.file_stem() else {
            return Err(format!("Invalid model file name: {}", base_path.display()));
        };

        let Some(stem) = stem_osstr.to_str() else {
            return Err("Model file name is not valid UTF-8".into());
        };

        let Ok(read_dir) = std::fs::read_dir(parent_dir) else {
            return Err(format!("Failed to read model directory: {}", parent_dir.display()));
        };

        let mut obj_paths: Vec<PathBuf> = read_dir
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let filename = path.file_name()?.to_str()?;
                if filename.starts_with(stem) && path.extension()?.to_str()? == "obj" {
                    Some(path)
                } else {
                    None
                }
            })
        .collect();

        obj_paths.sort();

        if obj_paths.is_empty() {
            return Err(format!("No .obj files found for model '{}'", name));
        }

        let lods: Vec<_> = obj_paths
            .into_iter()
            .map(|path| {
                self.runtime.block_on(async {
                    super::resources::load_model(
                        &path,
                        &graphics.device,
                        &graphics.queue,
                        texture_layout,
                    ).await
                })
            })
        .collect::<Result<Vec<_>, _>>()?;

        if lods.is_empty() {
            return Err(format!("Failed to load any LOD for model '{}'", name));
        }

        scene.add_render_object(model.clone(), RenderObject { lods });
        Ok(model)
    }
}
