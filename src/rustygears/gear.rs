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
use winit::event::WindowEvent;
use crate::Command;
use std::any::Any;
use winit::dpi::PhysicalSize;
use crate::ElementState;
use crate::KeyCode;
use crate::Game;

use crossbeam::channel::Sender;

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
///
///     fn keyboard_input(&mut self, key: KeyCode, state: ElementState, game: GameView) {
///         println!("Key {:?} is {:?}", key, state);
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
    fn setup(&mut self, game: &mut Game, sender: Sender<Box<dyn Command>>) {
        let _ = sender;
        let _ = game;
    }

    /// Called once per frame during the game’s update phase.
    ///
    /// Use this to perform per-frame logic such as AI, timers, or state transitions.
    ///
    /// # Parameters
    /// - `game`: A read-only snapshot of the current game state.
    fn update(&mut self, game: GameView ) {
        let _ = game;
    }

    /// Called when a mouse motion event occurs.
    ///
    /// # Parameters
    /// - `dx`: Horizontal mouse movement in pixels.
    /// - `dy`: Vertical mouse movement in pixels.
    /// - `game`: A read-only snapshot of the current game state.
    fn mouse_motion(&mut self, dx: f64, dy: f64, game: GameView) {
        let _ = dx;
        let _ = dy;
        let _ = game;
    }

    /// Called when a keyboard key is pressed or released.
    ///
    /// # Parameters
    /// - `key`: The keyboard key involved in the input event.
    /// - `state`: The state of the key (`Pressed` or `Released`).
    /// - `game`: A read-only snapshot of the current game state.
    fn keyboard_input(&mut self, key: KeyCode, state: ElementState, game: GameView) {
        let _ = key;
        let _ = state;
        let _ = game;
    }

    /// Called when a window-related event occurs (e.g. resize, focus, input method).
    ///
    /// # Parameters
    /// - `window_event`: The window event from winit.
    /// - `game`: A read-only snapshot of the current game state.
    fn window_event(&mut self, window_event: &WindowEvent, game: GameView) {
        let _ = window_event;
        let _ = game;
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
    fn exit(&mut self, game: GameView ) {
        let _ = game;
    }
}

/// Represents different types of events that can occur in the system.
/// Each variant corresponds to a specific action, such as updating, rendering, or handling user input.

#[derive(Debug, Clone)]
pub enum GearEvent {

    /// Dispatched when an update cycle occurs (e.g., physics or logic update).
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::Update() = event {
    ///     println!("Updating...");
    /// }
    /// ```
    Update(),

    /// Dispatched when the window is resized.
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::WindowResize(size) = event {
    ///     println!("New size: {}x{}", size.width, size.height);
    /// }
    /// ```
    WindowResize(PhysicalSize<u32>),

    /// Dispatched when a keyboard key is pressed or released.
    ///
    /// The event carries the key code and its state (`Pressed` or `Released`).
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::KeyboardInput(key, state) = event {
    ///     println!("Key: {:?}, State: {:?}", key, state);
    /// }
    /// ```
    KeyboardInput(KeyCode, ElementState),

    /// Dispatched when the mouse moves, providing the new coordinates.
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::MouseMotion(x, y) = event {
    ///     println!("Mouse moved: x = {}, y = {}", x, y);
    /// }
    /// ```
    MouseMotion(f64, f64),

    /// Dispatched for raw `winit` window events that do not fall under other categories.
    ///
    /// This allows low-level event handling, such as focus changes, mouse wheel input,
    /// DPI changes, etc.
    ///
    /// ### Example Usage
    /// ```rust
    /// use winit::event::WindowEvent;
    ///
    /// if let GearEvent::WindowEvent(WindowEvent::Focused(focused)) = event {
    ///     println!("Window focus: {}", focused);
    /// }
    /// ```
    WindowEvent(winit::event::WindowEvent),

    /// Signals that the gear thread should shut down gracefully.
    ///
    /// This is typically dispatched during engine shutdown or when dynamically removing a gear.
    /// Upon receiving this event, the gear should perform cleanup and exit its thread loop.
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::Exit() = event {
    ///     println!("Shutting down gear...");
    /// }
    /// ```
    Exit(),
}
