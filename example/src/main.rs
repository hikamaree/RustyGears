use rusty_gears::math::InnerSpace;
use rusty_gears::math::One;
use rusty_gears::math::Rotation3;
use rusty_gears::math::Quaternion;
use rusty_gears::math::Zero;
use rusty_gears::math::Deg;
use rusty_gears::math::vec3;
use rusty_gears::math::Vector3;
use rusty_gears::*;

pub struct CamSwitch;

impl Gear for CamSwitch {
    fn handle_event(&mut self, event: &GearEvent, game: &GameView, cmd: &mut CommandBuffer) {
        if let GearEvent::KeyboardInput(key, state) = event {
            if *key == KeyCode::KeyC && *state == ElementState::Pressed {
                let index = game.scene.active_camera_id().expect("no camera found");
                let id = (index) % game.scene.camera_count() + 1;
                cmd.spawn( SetDefaultCamera { id });
                println!("Switching camera: {} -> {}", index, id);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Spawner {
    pub h: f32,
    pub j: f32,
    pub k: f32,
    pub l: f32,

}

impl Gear for Spawner {
    fn handle_event(&mut self, event: &GearEvent, _game: &GameView, cmd: &mut CommandBuffer) {
        if let GearEvent::KeyboardInput(key, state) = event {
            if *key == KeyCode::KeyH && *state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(0.0, 30.0, -self.h),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                cmd.spawn( SpawnModel { 
                    file_path: "block.obj".to_string(),
                    transform,
                    render_tags: vec![RenderTag::PBR] 
                });

                self.h += 1.0;
            }

            if *key == KeyCode::KeyJ && *state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(-self.j, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                cmd.spawn( SpawnModel { 
                    file_path: "ball.obj".to_string(),
                    transform,
                    render_tags: vec![RenderTag::PBR] 
                });

                self.j += 1.0;
            }

            if *key == KeyCode::KeyK && *state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(self.k, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                cmd.spawn( SpawnModel { 
                    file_path: "ball.obj".to_string(),
                    transform,
                    render_tags: vec![RenderTag::PBR] 
                });

                self.k += 1.0;
            }

            if *key == KeyCode::KeyL && *state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(0.0, 30.0, self.l),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                cmd.spawn( SpawnModel { 
                    file_path: "block.obj".to_string(),
                    transform,
                    render_tags: vec![RenderTag::PBR] 
                });

                self.l += 1.0;
            }
        }
    }
}

fn custom_handle(camera: &mut Camera, event: &GearEvent, game: &GameView) {
    if let GearEvent::KeyboardInput(..) = event {
        if camera.get_id() == game.scene.active_camera_id().expect("no camera found") {
            println!("majmuneee");
        }
    }
}

pub fn main() {
    let camera1 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);
    let camera2 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);

    let mut camera3 = Camera::new((0.0, 0.0, 0.0), 0.0, 0.0);
    camera3.set_handle(custom_handle);

    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::new());
        game.add_gear("camwitch".into(), CamSwitch);
        game.add_gear("spawner".into(), Spawner::default());
        game.add_camera(camera1);
        game.add_camera(camera2);
        game.add_camera(camera3);
    }).setup(|game| {
        const SPACE_BETWEEN: f32 = 30.0;
        const NUM_INSTANCES_PER_ROW: usize = 10;

        for z in 0..NUM_INSTANCES_PER_ROW {
            for x in 0..NUM_INSTANCES_PER_ROW {
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

                let transform = Transform { 
                    position,
                    rotation,
                    scale: vec3(1.0, 1.0, 1.0)
                };

                game.spawn_model("semi.obj", transform, vec![RenderTag::PBR]);
            }
        }

        game.scene.add_gui(EngineStats::new());

        game.use_gear::<Spawner, _>("spawner".into(), |spawner| {
            spawner.j = 10.0;
            println!("majmuneee");
            println!("{}", spawner.h);
        });
    }).run();
}
