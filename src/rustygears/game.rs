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

use crate::Camera;
use crate::ModelInstance;
use crate::Command;
use crate::EguiRenderer;
use crate::Entity;
use crate::Transform;
use crate::BindGroupLayoutKey;
use crate::RenderObject;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Time;
use crate::GameView;
use crate::WorldScene;

use std::sync::Arc;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::thread::JoinHandle;
use std::path::Path;
use std::path::PathBuf;

use winit::event_loop::EventLoop;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

pub(crate) struct MouseDelta {
    pub dx: f64,
    pub dy: f64,
}

pub struct GearMessage {
    pub gear_event: GearEvent,
    pub game: GameView<'static>,
}

/// Represents the core application state, managing rendering, scene data, time progression,
/// and communication with worker threads ("gears").
///
/// The `Game` struct acts as the central access point for subsystems such as the ECS-based world scene,
/// graphics backend, and rendering pipelines. It also manages runtime model loading and entity spawning.
///
/// # Fields
/// - `setupfns`: Queue of initialization functions executed during game setup.
/// - `gear_channels`: Channels used to communicate with asynchronous worker systems ("gears").
/// - `gear_handles`: Join handles to running gear threads for lifecycle control.
/// - `command_receiver`: Channel for receiving commands from the game engine to be executed.
/// - `command_sender`: Channel for sending commands to the game engine.
/// - `graphics`: Interior-mutable reference to the GPU rendering context and pipeline state.
/// - `gui`: Optional immediate-mode GUI renderer (e.g., egui).
/// - `time`: Tracks timing, delta time, and frame progression.
/// - `scene`: Interior-mutable reference to the curr
pub struct Game {
    pub(crate) setupfns: VecDeque<Box<dyn FnOnce(&mut Game) + Send>>,
    pub(crate) gear_channels: HashMap<String, Sender<GearMessage>>,
    pub(crate) gear_handles: HashMap<String, JoinHandle<()>>,
    pub(crate) command_receiver: Receiver<Box<dyn Command>>,
    pub(crate) command_sender: Sender<Box<dyn Command>>,
    pub graphics: Arc<UnsafeCell<Option<Graphics>>>,
    pub gui: Option<EguiRenderer>,
    pub time: Time,
    pub(crate) scene: Arc<UnsafeCell<WorldScene>>,
    commands: VecDeque<Box<dyn Command>>,
    pub(crate) mouse_delta: MouseDelta,
}

impl Game {

    /// Creates a new `Game` instance with default components.
    ///
    /// # Returns
    /// A new `Game` instance.
    pub fn new() -> Self {
        let (command_sender, command_receiver) = crossbeam::channel::unbounded();

        Self {
            setupfns: VecDeque::new(),
            gear_channels: HashMap::new(),
            gear_handles: HashMap::new(),
            command_sender,
            command_receiver,
            graphics: Arc::new(UnsafeCell::new(None)),
            gui: None,
            time: Time::new(),
            scene: Arc::new(UnsafeCell::new(WorldScene::default())),
            commands: VecDeque::new(),
            mouse_delta: MouseDelta { dx: 0.0, dy: 0.0 }
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
    ///
    /// # Panics
    /// Will panic if the gear fails to receive events due to channel errors or thread issues.
    /// This function assumes gear event handling is fallible only in case of programmer error or gear crash.
    pub fn add_gear<T: Gear + 'static>(&mut self, id: String, mut gear: T) -> &mut Self {
        let setup_sender = self.command_sender.clone();
        gear.setup(self, setup_sender);
        let (gear_sender, gear_receiver) = crossbeam::channel::unbounded();
        self.gear_channels.insert(id.clone(), gear_sender.clone());

        let handle = std::thread::spawn(move || {
            while let Ok(msg) = gear_receiver.recv() {
                match msg.gear_event {
                    GearEvent::Update() => {
                        gear.update(msg.game);
                    },
                    GearEvent::MouseMotion(dx, dy) => {
                        gear.mouse_motion(dx, dy, msg.game);
                    },
                    GearEvent::KeyboardInput(key, state) => {
                        gear.keyboard_input(key, state, msg.game);
                    },
                    GearEvent::WindowEvent(ref window_event) => {
                        gear.window_event(&window_event, msg.game);
                    }
                    GearEvent::Exit() => {
                        gear.exit(msg.game);
                        break;
                    }
                    GearEvent::WindowResize(physical_size) => {
                        let _ = physical_size;
                    }
                }
            }
        });

        self.gear_handles.insert(id, handle);
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
        let graphics = match unsafe { & *self.graphics.get() } {
            Some(graphics) => graphics,
            None => return
        };

        let scene = unsafe { &*self.scene.get() };

        let time = self.time.clone();

        let mut dead_gears = Vec::new();

        for (id, sender) in &self.gear_channels {
            let gear_event = event.clone();

            let result = sender.send(GearMessage {
                gear_event,
                game: GameView {
                    graphics,
                    time,
                    scene,
                },
            });

            if let Err(_err) = result {
                dead_gears.push(id.clone());
            }
        }

        for id in dead_gears {
            self.gear_channels.remove(&id);
            self.gear_handles.remove(&id);
        }

        while let Ok(cmd) = self.command_receiver.try_recv() {
            self.commands.push_back(cmd);
        }
    }

    /// Returns a mutable reference to the active game scene (`WorldScene`).
    ///
    /// This function uses interior mutability via `UnsafeCell`, so it is marked unsafe internally.
    ///
    /// # Safety
    /// The caller must ensure no aliasing mutable references exist simultaneously.
    pub fn scene(&self) -> &mut WorldScene {
        unsafe { &mut *self.scene.get() }
    }

    /// Returns a mutable reference to the active graphics backend (`Graphics`).
    ///
    /// This uses `UnsafeCell` to access the interior mutable state safely in single-threaded context.
    ///
    /// # Panics
    /// Panics if the graphics backend has not been initialized (`None`).
    pub fn graphics(&self) -> Result<&mut Graphics, &'static str> {
        let graphics = unsafe { &mut *self.graphics.get() };
        match graphics {
            Some(graphics) => Ok(graphics),
            None => Err("graphics is not initialized")
        }
    }

    /// Updates camera matrices, GPU state, and timing for the current frame.
    ///
    /// This method:
    /// - Retrieves the active camera and updates its view/projection matrices.
    /// - Uploads camera information to the GPU.
    /// - Updates the global time system.
    pub(crate) fn update(&mut self) {
        self.dispatch_event(GearEvent::MouseMotion(self.mouse_delta.dx, self.mouse_delta.dy));
        self.mouse_delta.dx = 0.0;
        self.mouse_delta.dy = 0.0;

        self.time.update();

        let scene = unsafe { &mut *self.scene.get() };

        let Ok(graphics) = self.graphics() else {
            return;
        };

        let Some(camera_entity) = scene.active_camera else {
            return;
        };

        let final_transform = scene.get_camera_transform(camera_entity);


        if let Some(camera) = scene.world.get_mut::<Camera>(camera_entity) {
            camera.update_view_proj(&final_transform, &graphics.projection);
            graphics.update(camera);
        }

        while let Some(cmd) = self.commands.pop_front() {
            cmd.apply(self);
        }
    }

    /// Spawns a new model instance into the ECS world.
    ///
    /// If the model at `file_path` has not been previously loaded, this function attempts
    /// to load it first. Upon success, a new [`Entity`] is created with the given [`Transform`]
    /// and a [`ModelInstance`] component referencing the model.
    ///
    /// # Parameters
    /// - `file_path`: Relative path to the model file (e.g., `"tree/tree.obj"`).
    ///               This path is also used as the model's registration key.
    /// - `transform`: World-space transform to assign to the new entity.
    ///
    /// # Returns
    /// - `Ok(Entity)` if the model is successfully loaded (or already loaded) and the entity is spawned.
    /// - `Err(String)` if loading the model fails.
    ///
    /// # Errors
    /// This function returns an error if:
    /// - The model file path is invalid.
    /// - No LODs are found for the model.
    /// - Loading the model or GPU upload fails.
    /// - Required GPU resources (like texture layouts) are missing.
    pub fn spawn_model(&mut self, file_path: &str, transform: Transform) -> Result<Entity, String> {
        if !self.scene().render_objects.contains_key(file_path) {
            if let Err(err) = self.load_model(file_path) {
                return Err(format!("Failed to load model '{}': {}", file_path, err));
            }
        }

        let entity = self.scene().world.spawn();

        self.scene().world.insert(entity, transform);
        self.scene().world.insert(entity, ModelInstance {
            name: file_path.to_string(),
        });

        Ok(entity)
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
    pub fn load_model(&mut self, name: &str) -> Result<(), String> {
        let scene = unsafe { &mut *self.scene.get() };

        let Ok(graphics) = self.graphics() else {
            return Err("Graohics is not initialized".into());
        };

        let Some(texture_layout) = graphics.bind_group_layouts.get(&BindGroupLayoutKey::Texture) else {
            return Err("Missing texture bind group layout".into());
        };

        let Ok(rt) = tokio::runtime::Runtime::new() else {
            return Err("Failed to create Tokio runtime".into());
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
                rt.block_on(async {
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

        scene.add_render_object(name.to_string(), RenderObject { lods });
        Ok(())
    }
}
