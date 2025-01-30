use std::any::Any;

use rusty_gears::*;

pub struct CamSwitch;

impl Gear for CamSwitch {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let GearEvent::Input(window_event) = event {
            match window_event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state,
                            ..
                        },
                        ..
                } => {
                    if *key == KeyCode::KeyC && *state == ElementState::Pressed {
                        let index = game.get_camera_index();
                        game.set_active_camera((index + 1) % game.get_camera_cnt());
                    }
                }
                _ => {},
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub fn main() {
    Game::default()
        .add_gear(CamSwitch)
        .add_camera(Camera::new((0.0, 0.0, 0.0), Deg(0.0), Deg(0.0)))
        .run();
}
