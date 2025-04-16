use winit::dpi::PhysicalSize;
use crate::CommandBuffer;
use crate::ElementState;
use crate::KeyCode;
use super::GameView;

/// # Gear Trait and GearEvent Enum
///
/// The `Gear` trait represents an event-driven component that reacts to various types of [`GearEvent`]s.
/// It is designed for systems, game objects, or logic modules that need to respond to events in the game loop.
///
/// Implementors of this trait must define the [`handle_event`] method, which receives game events,
/// a read-only view of the game state, and a command buffer for scheduling state changes or actions.
///
/// ## Example Implementation
/// ```rust
/// pub struct ExampleGear;
///
/// impl Gear for ExampleGear {
///     fn handle_event(&mut self, event: &GearEvent, game: &GameView, commands: &mut CommandBuffer) {
///         if let GearEvent::Update() = event {
///             println!("update...");
///         }
///     }
/// }
/// ```
///
/// The `handle_event` method is invoked whenever a [`GearEvent`] is dispatched to the gear,
/// allowing it to inspect the current game state and optionally queue commands.
///
/// ## Usage
/// In the game loop, each active `Gear` receives events like `GearEvent::Update`, `GearEvent::Collision`, etc.
/// These events are passed along with a read-only [`GameView`] and a mutable reference to the [`CommandBuffer`].
///
/// ```rust
/// let mut gear = ExampleGear;
/// let event = GearEvent::Update();
/// gear.handle_event(&event, &game_view, &mut command_buffer);
/// ```
///
/// ## Thread Safety
/// All `Gear` types must be both `Send` and `Sync`, ensuring they can be safely shared or mutated across threads.

pub trait Gear: Send + Sync {

    /// Handles an incoming [`GearEvent`] by reading the game state and optionally queuing commands.
    ///
    /// # Parameters
    /// - `event`: A reference to the incoming event to handle.
    /// - `game`: A read-only view of the current game state.
    /// - `commands`: A command buffer used to queue changes or actions in response to the event.

    fn handle_event(&mut self, event: &GearEvent, game: &GameView, commands: &mut CommandBuffer);
}

/// Represents different types of events that can occur in the system.
/// Each variant corresponds to a specific action, such as updating, rendering, or handling user input.

#[derive(Clone)]
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

    /// Dispatched when a new frame render is requested.
    ///
    /// ### Example Usage
    /// ```rust
    /// if let GearEvent::RenderRequested() = event {
    ///     println!("Rendering...");
    /// }
    /// ```
    
    RenderRequested(),

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
}
