use winit::dpi::PhysicalSize;
use crate::CommandBuffer;
use crate::ElementState;
use crate::KeyCode;
use super::Game;

/// # Gear Trait and GearEvent Enum
///
/// The `Gear` trait represents an event-driven component that reacts to different types of `GearEvent` occurrences.
/// Implementors of this trait must define the `handle_event` function, which processes incoming events and updates the
/// component's state accordingly.
///
/// ## Example Implementation
/// ```rust
/// pub struct ExampleGear;
/// 
/// impl Gear for ExampleGear {
///     fn handle_event(&mut self, event: &GearEvent, _game: &mut Game) {
///         if let GearEvent::Update() = event {
///             println!("update...");
///         }
///     }
/// }
/// ```
///
/// The `handle_event` function is triggered whenever a `GearEvent` occurs, allowing `ExampleGear` to react accordingly.
///
/// ## Usage
/// When an event occurs in the game loop, it is dispatched to all active `Gear` components via `handle_event`. 
/// For example:
/// ```rust
/// let mut gear = ExampleGear;
/// let event = GearEvent::Update();
/// gear.handle_event(&event, &mut game);
/// ```

pub trait Gear: Send + Sync {
    /// Processes an incoming `GearEvent` and updates the `Gear` state accordingly.
    fn handle_event(&mut self, event: &GearEvent, game: &Game, commands: &mut CommandBuffer);
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
