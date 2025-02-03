use std::any::Any;
use winit::{event::ElementState, keyboard::KeyCode };
use crate::{Camera, Game};

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

impl Gear for Camera {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let Some(mut handler) = self.custom_handler.take() {
            handler(self, event, game);
            self.custom_handler = Some(handler);
            return;
        }

        if self.get_id() != game.get_camera_id() {
            return;
        }

        if let GearEvent::KeyboardInput(key, _state) = event {
            let dt = game.delta_time();

            match key {
                KeyCode::KeyW | KeyCode::ArrowUp => self.move_forward(dt),
                KeyCode::KeyS | KeyCode::ArrowDown => self.move_backward(dt),
                KeyCode::KeyA | KeyCode::ArrowLeft => self.move_left(dt),
                KeyCode::KeyD | KeyCode::ArrowRight => self.move_right(dt),
                _ => {},
            }
        }

        if let GearEvent::MouseMotion(x, y) = event {
            let dt = game.delta_time();
            self.rotate(*x as f32, *y as f32, dt);
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
