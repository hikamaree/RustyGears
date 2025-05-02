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

use crate::Command;
use crate::EguiRenderer;
use crate::RenderTag;
use crate::Transform;
use crate::Instance;
use crate::BindGroupLayoutKey;
use crate::RenderObject;
use crate::Gear;
use crate::GearEvent;
use crate::Graphics;
use crate::Scene;
use crate::Time;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::thread::JoinHandle;

use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use winit::event_loop::EventLoop;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use super::GameView;

pub struct MajmunskiEvent {
    pub gear_event: GearEvent,
    pub game: GameView,
}

pub struct Game {
    pub(crate) setupfns: VecDeque<Box<dyn FnOnce(&mut Game) + Send>>,
    pub(crate) gear_channels: HashMap<String, Sender<MajmunskiEvent>>,
    pub(crate) gear_handles: HashMap<String, JoinHandle<()>>,
    pub(crate) command_receiver: Receiver<Box<dyn Command>>,
    pub(crate) command_sender: Sender<Box<dyn Command>>,
    pub graphics: Option<Graphics>,
    pub gui: Option<EguiRenderer>,
    pub time: Time,
    pub scene: Scene,
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
            graphics: None,
            gui: None,
            time: Time::new(),
            scene: Scene::default(),
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
    /// where it listens for `MajmunskiEvent` messages. These events are dispatched from the main
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
                    GearEvent::Update() => gear.update(msg.game),
                    GearEvent::MouseMotion(dx, dy) => gear.mouse_motion(dx, dy, msg.game),
                    GearEvent::KeyboardInput(key, state) => gear.keyboard_input(key, state, msg.game),
                    GearEvent::WindowEvent(ref window_event) => gear.window_event(&window_event, msg.game),
                    _ => {}
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

    /// Dispatches a `GearEvent` to all registered gears in the game.
    ///
    /// For each gear, a `MajmunskiEvent` is constructed containing the event and a snapshot
    /// of the current game state (`GameView`), including `Scene`, `Time`, and `Graphics`.
    ///
    /// The event is sent to each gear via its dedicated communication channel.
    /// Each gear runs in its own thread and receives the event asynchronously.
    ///
    /// After dispatching, all pending commands returned by gears (sent via `Command`) are applied
    /// to the main game state immediately.
    ///
    /// # Arguments
    /// * `event` - The `GearEvent` to send to all gears.
    ///
    /// # Panics
    /// Will panic if sending on any gear channel fails (indicates a crashed gear thread).
    pub fn dispatch_event(&mut self, event: GearEvent) {
        self.gear_channels.par_iter().for_each(|(_, sender)| {
            sender.send(MajmunskiEvent {
                gear_event: event.clone(),
                game: GameView {
                    graphics: self.graphics.as_ref().unwrap().clone(),
                    time: self.time.clone(),
                    scene: self.scene.create_snapshot(),
                },
            }).unwrap();
        });

        while let Ok(cmd) = self.command_receiver.try_recv() {
            cmd.apply(self);
        }
    }



    pub fn spawn_model(&mut self, file_path: &str, transform: Transform, render_tags: Vec<RenderTag>) -> usize {
        if !self.scene.render_objects.contains_key(file_path) {
            self.load_model(file_path);
        }

        let instance = Instance::new(
            file_path.to_string(),
            transform,
            render_tags,
        );

        let id = instance.id();

        self.scene.add_instance(instance);

        id
    }

    pub fn load_model(&mut self, file_path: &str) {
        let graphics = self.graphics.as_ref().expect("ERROR: Graphics is not initialized");

        let texture_layout = graphics.bind_group_layouts.get(&BindGroupLayoutKey::Texture)
            .expect("Texture bind group layout not found");

        let rt = tokio::runtime::Runtime::new().unwrap();

        let model = rt.block_on(async {
            super::resources::load_model(
                file_path,
                &graphics.device,
                &graphics.queue,
                texture_layout,
            ).await
        });

        self.scene.add_render_object(file_path.to_string(), RenderObject { model });
    }
}
