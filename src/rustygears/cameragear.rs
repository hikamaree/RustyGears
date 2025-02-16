use crate::KeyCode;
use std::any::Any;
use crate::Game;
use crate::GearEvent;
use crate::Camera;
use crate::Gear;

impl Gear for Camera {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let Some(mut handler) = self.custom_handler.take() {
            handler(self, event, game);
            self.custom_handler = Some(handler);
            return;
        }

        if self.get_id() != game.cameras.active_camera_id().expect("no camera found") {
            return;
        }

        if let GearEvent::KeyboardInput(key, _state) = event {
            let dt = game.time.delta_time();

            match key {
                KeyCode::KeyW | KeyCode::ArrowUp => self.move_forward(dt),
                KeyCode::KeyS | KeyCode::ArrowDown => self.move_backward(dt),
                KeyCode::KeyA | KeyCode::ArrowLeft => self.move_left(dt),
                KeyCode::KeyD | KeyCode::ArrowRight => self.move_right(dt),
                _ => {},
            }
        }

        if let GearEvent::MouseMotion(x, y) = event {
            let dt = game.time.delta_time();
            self.rotate(*x as f32, *y as f32, dt);
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
