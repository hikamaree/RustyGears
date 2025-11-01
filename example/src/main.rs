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

use rusty_gears::math::Euler;
use rusty_gears::math::InnerSpace;
use rusty_gears::math::One;
use rusty_gears::math::Quaternion;
use rusty_gears::math::Rad;
use rusty_gears::math::Rotation;
use rusty_gears::math::Rotation3;
use rusty_gears::math::Vector3;
use rusty_gears::math::vec3;
use rusty_gears::*;

struct MyGame {
    pub truck: Entity,
    pub h: f32,
    pub camera1: Entity,
    pub camera2: Entity,
    pub block: Option<RenderObject>,
}

impl MyGame {
    fn new() -> Result<Self, String> {
        Ok(Self {
            truck: Entity::NULL,
            h: 0.0,
            camera1: Entity::NULL,
            camera2: Entity::NULL,
            block: None,
        })
    }

    async fn mouse_motion(&mut self, game: &GameView) {
        let Ok(input) = game.get::<Input>() else {
            return;
        };

        let mm = input.mouse_delta();

        let Ok(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Some(active_camera) = scene.active_camera else {
            return;
        };

        let sensitivity = 0.002;

        if active_camera == self.camera1 {
            let delta_yaw = Rad(-mm.dx as f32 * sensitivity);
            let delta_pitch = Rad(-mm.dy as f32 * sensitivity);

            FreeFlyCamera::update_rotation(self.camera1, delta_yaw, delta_pitch);
        } else if active_camera == self.camera2 {
            let delta_yaw = Rad(-mm.dx as f32 * sensitivity);
            let rot = Quaternion::from_angle_y(delta_yaw);

            let cam_handle = match scene.world.get::<CameraControl>(self.camera2) {
                Some(controller) => {
                    match controller.handler.as_any().downcast_ref::<CameraFollow>() {
                        Some(handle) => handle,
                        None => return,
                    }
                }
                None => return,
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
            let new_rotation_offset = Quaternion::from_angle_y(-Rad(flat_forward
                .z
                .atan2(flat_forward.x)
                + std::f32::consts::FRAC_PI_2));

            CameraFollow::set_position_offset(self.camera2, new_position_offset);
            CameraFollow::set_rotation_offset(self.camera2, new_rotation_offset);
        }
    }

    async fn keyboard_input(&mut self, game: &GameView) {
        let Ok(scene) = game.get::<WorldScene>() else {
            return;
        };

        let Ok(input) = game.get::<Input>() else {
            return;
        };

        let Ok(time) = game.get::<Time>() else {
            return;
        };

        let dt = time.delta_time();

        let speed = 50.0;

        let cam_handle = match scene.world.get::<CameraControl>(self.camera1) {
            Some(controller) => match controller.handler.as_any().downcast_ref::<FreeFlyCamera>() {
                Some(handle) => handle,
                None => return,
            },
            None => return,
        };

        let forward = cam_handle.forward();
        let right = cam_handle.right();

        if input.pressed(KeyCode::KeyX) {
            game.load_obj_model("scene1/scene1.obj");
        }

        if input.pressed(KeyCode::KeyC) {
            let Some(active_camera) = scene.active_camera else {
                return;
            };

            let camera = if active_camera == self.camera1 {
                self.camera2
            } else {
                self.camera1
            };

            game.set_default_camera(&camera);
        }

        if input.active(KeyCode::ArrowUp) {
            game.update_entity_position(&self.truck, Vector3::new(0.0, 0.0, speed * dt));
        }

        if input.active(KeyCode::ArrowDown) {
            game.update_entity_position(&self.truck, Vector3::new(0.0, 0.0, -speed * dt));
        }

        if input.active(KeyCode::KeyW) {
            FreeFlyCamera::update_position(self.camera1, forward * speed * dt);
        }

        if input.active(KeyCode::KeyS) {
            FreeFlyCamera::update_position(self.camera1, -forward * speed * dt);
        }

        if input.active(KeyCode::KeyA) {
            FreeFlyCamera::update_position(self.camera1, -right * speed * dt);
        }

        if input.active(KeyCode::KeyD) {
            FreeFlyCamera::update_position(self.camera1, right * speed * dt);
        }

        if input.active(KeyCode::KeyE) {
            self.h += 1.0;
            let transform = Transform {
                position: vec3(0.0, 30.0, -self.h),
                rotation: Quaternion::one(),
                scale: vec3(1.0, 1.0, 1.0),
            };

            if let Some(block) = self.block.clone() {
                game.spawn_entity((transform, block));
            }
        }
    }
}

impl Gear for MyGame {
    async fn setup(&mut self, game: &GameView) {
        let transform = Transform {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0),
        };

        self.truck = Entity::new().insert(transform);

        self.camera1 = game.spawn_entity((
            Camera::new(),
            Transform::identity(),
            CameraControl {
                handler: Box::new(FreeFlyCamera::new()),
            },
        ));
        game.set_default_camera(&self.camera1);

        self.camera2 = game.spawn_entity((
            Camera::new(),
            CameraControl {
                handler: Box::new(CameraFollow {
                    target: self.truck,
                    position_offset: vec3(0.0, 20.0, -60.0),
                    rotation_offset: Quaternion::one(),
                }),
            },
        ));

        let block = game.load_obj_model("ball/ball.obj");
        self.block = Some(RenderObject { lods: vec![block] });

        let truck_lod0 = game.load_obj_model("truck/semi.obj");

        let truck_lod1 = game.load_obj_model("truck/semi_lod1.obj");

        let truck_lod2 = game.load_obj_model("truck/semi_lod2.obj");

        self.truck.insert(RenderObject {
            lods: vec![truck_lod0, truck_lod1, truck_lod2],
        });

        // miku //

        let miku_lod0 = game.load_obj_model("miku/miku.obj");

        let miku_lod1 = game.load_obj_model("miku/miku_lod1.obj");

        let miku_lod2 = game.load_obj_model("miku/miku_lod2.obj");

        const SPACE_BETWEEN: f32 = 15.0;
        const NUM_INSTANCES_PER_ROW: usize = 50;

        let positions = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x_pos = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z_pos = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                vec3(x_pos, 0.0, z_pos)
            })
        });

        for position in positions {
            let transform = Transform {
                position,
                rotation: Quaternion::one(),
                scale: vec3(300.0, 300.0, 300.0),
            };

            let miku = RenderObject {
                lods: vec![miku_lod0.clone(), miku_lod1.clone(), miku_lod2.clone()],
            };

            game.spawn_entity((miku.clone(), transform));
        }

        // random scene //

        let transform = Transform {
            position: vec3(0.0, 0.0, 40.0),
            rotation: Quaternion::one(),
            scale: vec3(7.0, 7.0, 7.0),
        };

        let scene3 = game.load_obj_model("scene3/scene3.obj");
        game.spawn_entity((RenderObject { lods: vec![scene3] }, transform));

        let transform = Transform {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0),
        };

        let scene1 = game.load_obj_model("scene1/scene1.obj");
        let s1e = game.spawn_entity((RenderObject { lods: vec![scene1] }, transform));

        log!(LogKind::Info, "scene1 entity = {:?}", s1e);

        //landscape//

        log!(LogKind::Info, "loading landscape");

        let Ok(landscape) = Landscape::from_heightmap("resources/heightmap.png", 1.0, 10.0) else {
            log!(LogKind::Error, "failed to make landscape");
            return;
        };

        let Ok(landscape_mesh) = landscape.clone().generate_model(game) else {
            log!(LogKind::Error, "failed to make mesh for landscape");
            return;
        };

        let landscape_model3d = game.add_model3d("landscape", landscape_mesh);

        let entity = Entity::new()
            .insert(RenderObject {
                lods: vec![landscape_model3d],
            })
            .insert(landscape)
            .insert(Transform::identity());

        log!(LogKind::Info, "landscape loaded as {:?}", entity);

        // lights //

        let big_block = game.load_obj_model("kocka/kocka.obj");
        let bb_render = RenderObject {
            lods: vec![big_block],
        };

        Entity::new()
            .insert(bb_render)
            .insert(Transform {
            position: vec3(0.0, 180.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(50.0, 1.0, 50.0),
        });

        Entity::new()
            .insert(PointLight {
                color: [1.0, 1.0, 1.0],
                intensity: 2.0,
                radius: 20.0,
            })
            .insert(Transform {
                position: vec3(-15.0, 200.0, 0.0),
                rotation: Quaternion::one(),
                scale: vec3(1.0, 1.0, 1.0),
            })
            .insert(self.block.clone().unwrap());

        Entity::new()
            .insert(DirectionalLight {
                color: [0.0, 1.0, 0.0],
                intensity: 0.1,
            })
            .insert(Transform {
                position: vec3(0.0, 200.0, 20.0),
                rotation: Quaternion::from(Euler {
                    x: Rad(std::f32::consts::FRAC_PI_2),
                    y: Rad(0.0),
                    z: Rad(0.0),
                }),
                scale: vec3(1.0, 1.0, 1.0),
            })
            .insert(self.block.clone().unwrap());

        Entity::new()
            .insert(SpotLight {
                color: [0.0, 0.0, 1.0],
                intensity: 1.0,
                radius: 40.0,
                inner_angle: 10f32.to_radians(),
                outer_angle: 20f32.to_radians(),
            })
            .insert(Transform {
                position: vec3(15.0, 200.0, -5.0),
                rotation: Quaternion::from(Euler {
                    x: Rad(std::f32::consts::FRAC_PI_4),
                    y: Rad(-std::f32::consts::FRAC_PI_4),
                    z: Rad(0.0),
                }),
                scale: vec3(1.0, 1.0, 1.0),
            })
            .insert(self.block.clone().unwrap());
    }

    async fn update(&mut self, game: GameView) {
        self.mouse_motion(&game).await;
        self.keyboard_input(&game).await;
    }
}

#[tokio::main]
pub async fn main() {
    Game::new()
        .setup(|game| {
            game.add_gear("render".into(), Render::default());
            let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
                return;
            };
            scene.add_gui(EngineStats::new());
            scene.add_gui(TerminalGui::new(default_terminal_commands()));
        })
        .setup(|game| {
            if let Ok(mygame) = MyGame::new() {
                game.add_gear("mygame".into(), mygame);
            }
        })
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_access_is_safe() {
        let mut game = Game::new()
            .setup(|game| {
                if let Ok(mygame) = MyGame::new() {
                    game.add_gear("mygame".into(), mygame);
                }
            })
            .setup(|game| {
                if let Err(err) = game.components.with::<WorldScene, _>(|scene| {
                    scene.add_gui(EngineStats::new());
                }) {
                    log!(
                        LogKind::Error,
                        "Failed to add EngineStats to scene: {}",
                        err
                    );
                }
            });

        for _ in 0..10 {
            game.dispatch_event(GearEvent::Update);
        }
    }
}
