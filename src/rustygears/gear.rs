use crate::ElementState;
use crate::KeyCode;
use crate::Game;
use std::any::Any;

///Gear implementation example
/// ```rust
///pub struct ExampleGear;
/// 
///impl Gear for ExampleGear {
///    fn handle_event(&mut self, event: &GearEvent, _game: &mut Game) {
///        if let GearEvent::Update() = event {
///            println!("update...");
///        }
///    }
///
///    fn as_any(&self) -> &dyn Any { self }
///    fn as_any_mut(&mut self) -> &mut dyn Any { self }
///}
///```

pub trait Gear: Send + Sync {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


#[derive(Clone)]
pub enum GearEvent {

///```rust
/// if let GearEvent::Update() = event {
///     println!("Updating...");
/// }
///```

    Update(),

///```rust
/// if let GearEvent::RenderRequested() = event {
///     println!("Rendering...");
/// }
///```

    RenderRequested(),

///```rust
/// if let GearEvent::KeyboardInput(key, state) = event {
///         println!("Key: {:?}, State: {:?}", key, state);
///     }
/// }
///```

    KeyboardInput(KeyCode, ElementState),

///```rust
/// else if let GearEvent::MouseMotion(x, y) = event {
///     println!("Mouse moved: x = {}, y = {}", x, y);
/// }
///```

    MouseMotion(f64, f64),

}
