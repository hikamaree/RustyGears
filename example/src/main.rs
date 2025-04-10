// use hecs::CommandBuffer;
use cgmath::InnerSpace;
use rusty_gears::*;

pub struct CamSwitch;

// impl Gear for CamSwitch {
//     fn handle_event(&mut self, event: &GearEvent, game: &GameView, _cmd: CommandBuffer) {
//         if let GearEvent::KeyboardInput(key, state) = event {
//             if *key == KeyCode::KeyC && *state == ElementState::Pressed {
//                 let index = game.cameras.active_camera_id().expect("no camera found");
//                 game.cameras.set_active_camera((index + 1) % game.cameras.count() + 1);
//                 println!("{}", index);
//             }
//         }
//     }
// }

fn custom_handle(camera: &mut Camera, event: &GearEvent, game: &Game) {
    if let GearEvent::KeyboardInput(..) = event {
        if camera.get_id() == game.cameras.active_camera_id().expect("no camera found") {
            println!("majmuneee");
        }
    }
}

pub fn main() {
    let camera1 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);
    // let camera2 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);

    let mut camera3 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);
    camera3.set_handle(custom_handle);


    GameBuilder::new().setup(|game| {
        game.add_gear(Render::new(&game.graphics));
        game.add_gear(EngineStats::new());
        game.add_camera(camera1);
    }).setup(|game| {
        const SPACE_BETWEEN: f32 = 30.0;
        const NUM_INSTANCES_PER_ROW: usize = 10;
        let transforms = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        Quaternion::from_axis_angle(
                            Vector3::unit_z(),
                            Deg(0.0),
                        )
                    } else {
                        Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                    };

                    Transform { position, rotation, scale: vec3(1.0, 1.0, 1.0) }
                })
            })
            .collect::<Vec<_>>();


        for transform in transforms {
            if let Err(e) = game.spawn_model("semi.obj", transform, vec![RenderTag::PBR]) {
                eprintln!("Failed to spawn semi.obj model: {}", e);
            }
        }




    }).run();
}
