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

use crate::GameView;
use std::any::Any;

/// # Gear Trait and GearEvent Enum
///
/// The `Gear` trait represents an event-driven component in the game engine. 
/// It is intended for systems, gameplay logic, or objects that need to react to high-level events 
/// such as updates, input, or window interactions.
///
/// Each gear is run in its own thread and communicates via event messages. 
/// Events are received via the `GearEvent` enum and handled by the appropriate method (e.g., `update`, `mouse_motion`, etc.).
///
/// ## Example Implementation
/// ```rust
/// pub struct ExampleGear;
///
/// impl Gear for ExampleGear {
///     fn update(&mut self, game: GameView) {
///         println!("Game updated. Delta time: {:?}", game.time.delta_time());
///     }
/// }
/// ```
///
/// Gears interact with the game only through the provided [`GameView`] (read-only snapshot)
/// and can issue actions through the `CommandBuffer` during setup.
///
/// ## Thread Safety
/// All gears must be `Send + Sync + 'static` to be safely run in their own threads and receive event messages.
///
/// ## Lifecycle
/// - `setup` is called on the main thread before the gear starts running.
/// - All other methods (`update`, `mouse_motion`, etc.) are called in the gear’s thread in response to dispatched events.
pub trait Gear: Any + Send + Sync {

    /// Called once when the gear is being initialized or added to the game.
    ///
    /// Use this method to register entities, set up initial state, or schedule startup commands.
    ///
    /// # Parameters
    /// - `game`: A mutable reference to the game state during setup. Use this to directly modify entities or world state.
    /// - `sender`: A command sender for scheduling actions via `Command`. Can be cloned and stored for future use.
    fn setup(&mut self, game: &GameView) -> impl std::future::Future<Output = ()> + Send {
        async {
            let _ = game;
        }
    }

    /// Called once per frame during the game’s update phase.
    ///
    /// Use this to perform per-frame logic such as AI, timers, or state transitions.
    ///
    /// # Parameters
    /// - `game`: A read-only snapshot of the current game state.
    fn update(&mut self, game: GameView) -> impl std::future::Future<Output = ()> + Send {
        async {
            let _ = game;
        }
    }

    /// Called when the gear is being shut down or removed from the game.
    ///
    /// This method provides an opportunity to perform any necessary cleanup,
    /// such as deallocating resources, saving state, or sending final commands.
    ///
    /// It is guaranteed to be called exactly once before the gear's thread exits,
    /// if the `Exit` event is dispatched via `GearEvent::Exit`.
    ///
    /// # Parameters
    /// - `game`: A read-only snapshot of the current game state at shutdown time.
    fn exit(&mut self, game: GameView ) -> impl std::future::Future<Output = ()> + Send {
        async {
            let _ = game;
        }
    }
}

/// Represents different types of events that can occur in the system.
/// Each variant corresponds to a specific action, such as updating, rendering, or handling user input.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum GearEvent {
    /// Dispatched when an update cycle occurs (e.g., physics or logic update).
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::Update = event {
    ///     println!("Updating...");
    /// }
    /// ```
    Update,

    /// Signals that the gear thread should shut down gracefully.
    ///
    /// This is typically dispatched during engine shutdown or when dynamically removing a gear.
    /// Upon receiving this event, the gear should perform cleanup and exit its thread loop.
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::Exit = event {
    ///     println!("Shutting down gear...");
    /// }
    /// ```
    Exit,
}

/// Message sent to a gear thread containing an event and a view of the game state.
///
/// `GearMessage` is used internally by the engine to communicate between the main thread and
/// background "gear" systems. Each message encapsulates a specific [`GearEvent`] to be processed,
/// as well as a lightweight [`GameView`] allowing read-only access to shared components.
///
/// Gear systems receive these messages via a channel and handle them by reacting to the event kind.
///
/// # Fields
/// - `gear_event`: The event to be handled by the gear system, such as `Update`, `Exit` etc.
/// - `game`: A snapshot of the current game state (`GameView`) with access to components and scene data.
///
/// # See also
/// - [`Gear`]: The trait implemented by gear systems that receive and process these messages.
/// - [`GameView`]: A read-only handle to shared game state components.
/// - [`GearEvent`]: Enum describing the type of gear-related event being dispatched.
pub struct GearMessage {
    pub gear_event: GearEvent,
    pub game: GameView,
}
