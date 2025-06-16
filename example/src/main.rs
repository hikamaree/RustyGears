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

use rusty_gears::math::InnerSpace;
use rusty_gears::math::Rotation;
use rusty_gears::math::Rotation3;
use rusty_gears::math::Vector3;
use rusty_gears::math::Rad;
use rusty_gears::math::One;
use rusty_gears::math::Quaternion;
use rusty_gears::math::vec3;
use rusty_gears::*;

struct MyGame {
    pub sender: Option<Sender<Box<dyn Command>>>,
    pub truck: Entity,
    pub h: f32,
    pub camera1: Entity,
    pub camera2: Entity,
    pub block: Model3d,
}

impl MyGame {
    fn new(game: &mut Game) -> Self {
        let truck = match game.load_model("truck/semi") {
            Ok(truck) => truck,
            Err(e) => {
                println!("{}", e);
                todo!()
            },
        };

        let scene = match game.components.get_mut::<WorldScene>() {
            Ok(scene) => scene,
            Err(_) => todo!(),
        };

        let transform = Transform {
            position: vec3(8.0, -10.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0) 
        };

        let truck = scene.spawn()
            .with(truck)
            .with(transform)
            .build();

        let camera1 = scene.spawn()
            .with(Camera::new())
            .with(Transform::identity())
            .with(CameraControl {
                handler: Box::new(FreeFlyCamera::new()),
            })
            .build();

        scene.set_active_camera(camera1);

        let camera2 = scene.spawn()
            .with(Camera::new())
            .with(CameraControl {
                handler: Box::new(CameraFollow{
                    target: truck,
                    position_offset: vec3(0.0, 20.0, -60.0),
                    rotation_offset: Quaternion::one()
                })
            }).build();

        let block = match game.load_model("truck/semi") {
            Ok(truck) => truck,
            Err(_) => todo!(),
        };

        Self {
            sender: None,
            truck, 
            h: 0.0,
            camera1,
            camera2,
            block,
        }
    }

    fn mouse_motion(&mut self, dx: f64, dy: f64, game: &GameView) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let Some(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Some(active_camera) = scene.active_camera else {
            return;
        };

        let sensitivity = 0.005;

        if active_camera == self.camera1 {
            let sensitivity = 0.002;
            let delta_yaw = Rad(-dx as f32 * sensitivity);
            let delta_pitch = Rad(-dy as f32 * sensitivity);

            update_freefly_rotation!(sender, self.camera1, delta_yaw, delta_pitch);
        } else if active_camera == self.camera2 {

            let delta_yaw = Rad(-dx as f32 * sensitivity);
            let rot = Quaternion::from_angle_y(delta_yaw);

            let cam_handle = match scene.world.get::<CameraControl>(self.camera2) {
                Some(controller) => match controller.handler.as_any().downcast_ref::<CameraFollow>() {
                    Some(handle) => handle,
                    None => return
                },
                None => return
            };

            let Some(target_transform) = scene.world.get::<Transform>(cam_handle.target) else {
                return;
            };

            let new_position_offset = rot.rotate_vector(cam_handle.position_offset);

            let camera_position = target_transform.position + new_position_offset;

            let mut flat_forward = target_transform.position - camera_position;
            flat_forward.y = 0.0;

            if flat_forward.magnitude2() < 0.0001 {
                return;
            }

            let flat_forward = flat_forward.normalize();
            let new_rotation_offset = Quaternion::from_angle_y(
                -Rad(flat_forward.z.atan2(flat_forward.x) + std::f32::consts::FRAC_PI_2),
            );

            set_camerafollow_position_offset!(sender, self.camera2, new_position_offset);
            set_camerafollow_rotation_offset!(sender, self.camera2, new_rotation_offset);
        }
    }


    fn keyboard_input(&mut self, input: &Input, game: &GameView) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let Some(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Some(time) = game.get::<Time>() else {
            return;
        };

        let dt = time.delta_time();

        let speed = 50.0;

        let cam_handle = match scene.world.get::<CameraControl>(self.camera1) {
            Some(controller) => match controller.handler.as_any().downcast_ref::<FreeFlyCamera>() {
                Some(handle) => handle,
                None => return
            },
            None => return
        };

        let forward = cam_handle.forward();
        let right = cam_handle.right();

        if input.is_key_pressed(KeyCode::KeyC) {
            let Some(active_camera) = scene.active_camera else {
                return;
            };

            let camera = if active_camera == self.camera1 {
                self.camera2
            } else {
                self.camera1
            };

            set_default_camera!(sender, camera);
        }

        if input.is_key_pressed(KeyCode::ArrowUp) {
            update_entity_position!(sender, self.truck, Vector3::new(0.0, 0.0, speed * dt));
        }

        if input.is_key_pressed(KeyCode::ArrowDown) {
            update_entity_position!(sender, self.truck, Vector3::new(0.0, 0.0, -speed * dt));
        }

        if input.is_key_pressed(KeyCode::KeyW) {
            update_freefly_position!(sender, self.camera1, forward * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyS) {
            update_freefly_position!(sender, self.camera1, -forward * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyA) {
            update_freefly_position!(sender, self.camera1, -right * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyD) {
            update_freefly_position!(sender, self.camera1, right * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyE) {
            self.h += 1.0;
            let transform = Transform {
                position: vec3(0.0, 30.0, -self.h),
                rotation: Quaternion::one(),
                scale: vec3(1.0, 1.0, 1.0) 
            };

            let x = spawn_entity!(sender, transform);
            add_component!(sender, x, self.block.clone());
        }
    }
}

impl Gear for MyGame {
    fn setup(&mut self, game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);

        let miku = match game.load_model("miku/miku") {
            Ok(miku) => {
                miku
            },
            Err(e) => {
                eprintln!("{}", e);
                todo!()
            },
        };

        let Ok(scene) = game.components.get_mut::<WorldScene>() else {
            return;
        };

        const SPACE_BETWEEN: f32 = 15.0;
        const NUM_INSTANCES_PER_ROW: usize = 64;

        for z in 0..NUM_INSTANCES_PER_ROW {
            for x in 0..NUM_INSTANCES_PER_ROW {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = vec3(x, -10.0, z);

                let rotation = Quaternion::one();

                let transform = Transform { 
                    position,
                    rotation,
                    scale: vec3(300.0, 300.0, 300.0)
                };

                scene.spawn()
                    .with(miku.clone())
                    .with(transform);
                }
        }
    }

    fn update(&mut self, game: GameView) {
        let Some(input) = game.get::<Input>() else {
            return;
        };

        let mm = input.mouse_delta();

        self.mouse_motion(mm.dx, mm.dy, &game);

        self.keyboard_input(input, &game);
    }
}

pub fn main() {
    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::default());
        let mygame = MyGame::new(game);
        game.add_gear("mygame".into(), mygame);
    }).setup(|game| {
        let Ok(scene) = game.components.get_mut::<WorldScene>() else {
            return;
        };
        scene.add_gui(EngineStats::new());

        // let transform = Transform { 
        //     position: vec3(0.0, 0.0, 0.0),
        //     rotation: Quaternion::one(),
        //     scale: vec3(1.0, 1.0, 1.0)
        // };
        //
        // if let Err(e) = game.spawn_model("scene1/scene1", transform) {
        //     eprintln!("{}", e);
        // }
        //
        // let transform = Transform { 
        //     position: vec3(40.0, 0.0, 0.0),
        //     rotation: Quaternion::one(),
        //     scale: vec3(0.5, 0.5, 0.5)
        // };
        //
        // if let Err(e) = game.spawn_model("scene2/scene2", transform) {
        //     eprintln!("{}", e);
        // }
        //
        // let transform = Transform { 
        //     position: vec3(0.0, 0.0, 40.0),
        //     rotation: Quaternion::one(),
        //     scale: vec3(7.0, 7.0, 7.0)
        // };
        //
        // if let Err(e) = game.spawn_model("scene3/scene3", transform) {
        //     eprintln!("{}", e);
        // }
        //
        // let transform = Transform { 
        //     position: vec3(25.0, 0.0, 20.0),
        //     rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Rad(std::f32::consts::PI)),
        //     scale: vec3(0.3, 0.3, 0.3)
        // };
        //
        // if let Err(e) = game.spawn_model("scene4/scene4", transform) {
        //     eprintln!("{}", e);
        // }
    }).run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_access_is_safe() {
        let mut game = Game::new()
            .setup(|game| {
                let mygame = MyGame::new(game);
                game.add_gear("mygame".into(), mygame);

            }).setup(|game| {
                if let Err(err) = game.components.with::<WorldScene, _>(|scene| {
                    scene.add_gui(EngineStats::new());
                }) {
                    eprintln!("Failed to add EngineStats to scene: {}", err);
                }
            });

        for _ in 0..10 {
            game.dispatch_event(GearEvent::MouseMotion(10.0, 10.0));
            game.dispatch_event(GearEvent::Update());
        }
    }
}
