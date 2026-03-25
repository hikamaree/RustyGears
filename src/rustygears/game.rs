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
use crate::Command;
use crate::Gear;
use crate::GearEvent;
use crate::Time;
use crate::GameView;
use crate::WorldScene;

use std::collections::HashMap;
use std::collections::VecDeque;

use winit::event_loop::EventLoop;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

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

        let mut components = ComponentMap::new();

        components.insert(WorldScene::default());
        components.insert(Time::new());
        components.insert(Input::new());
        components.insert(Logs::default());

        Self {
            setupfns: VecDeque::new(),
            gear_channels: HashMap::new(),
            command_receiver,
            components,
        }
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
    /// * `gear` - An instance of a type that implements the [`Gear`] trait.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.
    pub fn add_gear<T: Gear + Send + Sync + 'static>(&mut self, id: String, gear: T) -> &mut Self {
        self.add_gear_impl(id, gear, false)
    }

    /// Adds a new gear synchronously, waiting for setup to complete before returning.
    ///
    /// Unlike [`add_gear`][Self::add_gear], this method blocks until the gear's [`Gear::setup`]
    /// method has finished executing. Any commands sent during setup are also processed
    /// before this method returns.
    ///
    /// Use this for gears that must be fully initialized before other setup functions run,
    /// such as the Render gear which needs to create GPU resources before the game loop starts.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for the gear.
    /// * `gear` - An instance of a type that implements the [`Gear`] trait.
    ///
    /// # Returns
    /// A mutable reference to the `Game` instance to allow method chaining.
    pub fn add_gear_sync<T: Gear + Send + Sync + 'static>(&mut self, id: String, gear: T) -> &mut Self {
        self.add_gear_impl(id, gear, true)
    }

    fn add_gear_impl<T: Gear + Send + Sync + 'static>(&mut self, id: String, mut gear: T, wait_for_setup: bool) -> &mut Self {
        let gameview = GameView::new(self.components.clone());
        let handle = tokio::runtime::Handle::current();
        
        let (setup_done_tx, setup_done_rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            handle.spawn(async move {
                gear.setup(&gameview).await;
                let _ = setup_done_tx.send(());
                crate::log!(crate::LogKind::Info, "Gear {} is ready", id);

                let (gear_sender, gear_receiver) = crossbeam::channel::unbounded();
                crate::send_command(GearSetupFinished {
                    id: id.clone(),
                    gear_channel: gear_sender,
                });

                while let Ok(msg) = gear_receiver.recv() {
                    let start = std::time::Instant::now();
                    match msg.gear_event {
                        GearEvent::Update => {
                            gear.update(msg.game).await;
                        },
                        GearEvent::Exit => {
                            gear.exit(msg.game).await;
                        }
                    }
                    let duration = start.elapsed().as_micros() as f32;
                    crate::send_command(UpdateDoneCommand { id: id.clone(), duration } );
                    if msg.gear_event == GearEvent::Exit { break; }
                }
            });
        });

        if wait_for_setup {
            setup_done_rx.recv().unwrap();
            while let Ok(cmd) = self.command_receiver.try_recv() {
                cmd.apply(self);
            }
        }

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
    /// game.dispatch_event(GearEvent::Update);
    /// ```
    pub fn dispatch_event(&mut self, event: GearEvent) {
        let mut dead_gears = Vec::new();

        let mut pending_gear_updates = 0;

        for (id, sender) in &self.gear_channels {
            let gear_event = event.clone();

            let result = sender.send(GearMessage {
                gear_event,
                game: GameView::new(self.components.clone())
            });

            if let Err(_err) = result {
                dead_gears.push(id.clone());
            } else {
                pending_gear_updates += 1;
            }
        }

        for id in dead_gears {
            self.gear_channels.remove(&id);
        }

        while self.gear_channels.len() == 0 {
            if let Ok(cmd) = self.command_receiver.recv() {
                cmd.apply(self);
            }
        }

        let mut commands: Vec<Box<dyn Command>> = vec![];

        while pending_gear_updates > 0 {
            if let Ok(cmd) = self.command_receiver.recv() {
                if (&*cmd as &dyn std::any::Any).downcast_ref::<UpdateDoneCommand>().is_some() {
                    pending_gear_updates -= 1;
                    cmd.apply(self);
                } else {
                    commands.push(cmd);
                }
            }
        }

        commands.sort_by_key(|c| c.priority());
        for cmd in commands {
            cmd.apply(self);
        }

        if event == GearEvent::Exit {
            std::process::exit(0);
        }
    }
}

struct GearSetupFinished {
    pub id: String,
    pub gear_channel: Sender<GearMessage>,
}

impl Command for GearSetupFinished {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.gear_channels.insert(self.id.clone(), self.gear_channel);
        println!("added gear {}", self.id);
    }
}

struct UpdateDoneCommand {
    pub id: String,
    pub duration: f32,
}

impl crate::Command for UpdateDoneCommand {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Ok(mut time) = game.components.get_mut::<crate::Time>() {
            time.set_gear_update_time(&self.id, self.duration);
        }
    }
}
