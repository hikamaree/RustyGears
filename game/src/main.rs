use std::any::Any;

use rusty_gears::*;

pub struct CamSwitch;

impl Gear for CamSwitch {
    fn handle_event(&mut self, event: &GearEvent, game: &mut Game) {
        if let GearEvent::KeyboardInput(key, state) = event {
            if *key == KeyCode::KeyC && *state == ElementState::Pressed {
                let index = game.cameras.active_camera_id().expect("no camera found");
                game.cameras.set_active_camera((index + 1) % game.cameras.count() + 1);
                println!("{}", index);
            }
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

fn custom_handle(camera: &mut Camera, event: &GearEvent, game: &mut Game) {
    if let GearEvent::KeyboardInput(..) = event {
        if camera.get_id() == game.cameras.active_camera_id().expect("no camera found") {
            println!("majmuneee");
        }
    }
}

pub fn main() {
    let camera1 = Camera::new((0.0, 0.0, 0.0), Deg(0.0), Deg(0.0));
    let camera2 = Camera::new((0.0, 0.0, 0.0), Deg(0.0), Deg(0.0));

    let mut camera3 = Camera::new((0.0, 0.0, 0.0), Deg(0.0), Deg(0.0));
    camera3.set_handle(custom_handle);

    Game::new()
        .add_gear(CamSwitch)
        .add_camera(camera1)
        .add_camera(camera2)
        .add_camera(camera3)
        .run();
}
