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
use std::sync::Arc;

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
    pub runtime: Arc<Runtime>,
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

        let runtime: Arc<Runtime> = tokio::runtime::Runtime::new().expect("ERROR: Failed to create tokio runtime").into();

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
        let (gear_sender, gear_receiver) = crossbeam::channel::unbounded();
        self.gear_channels.insert(id.clone(), gear_sender.clone());

        let gameview = GameView {
            components: self.components.get_view(),
        };

        let runtime = self.runtime.clone();

        std::thread::spawn(move || {
            runtime.block_on(async move {
                gear.setup(&gameview).await;
                while let Ok(msg) = gear_receiver.recv() {
                    match msg.gear_event {
                        GearEvent::Update() => {
                            gear.update(msg.game).await;
                            crate::send_command(super::UpdateDoneCommand);
                        },
                        GearEvent::Exit() => {
                            gear.exit(msg.game).await;
                            crate::send_command(super::UpdateDoneCommand);
                            break;
                        }
                    }
                }
            })
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

        while pending_gear_updates > 0 {
            if let Ok(cmd) = self.command_receiver.recv() {
                if (&*cmd as &dyn std::any::Any).downcast_ref::<crate::UpdateDoneCommand>().is_some() {
                    pending_gear_updates -= 1;
                } else {
                    cmd.apply(self);
                }
            }
        }
    }
}
