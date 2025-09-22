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
    pub truck: Entity,
    pub h: f32,
    pub camera1: Entity,
    pub camera2: Entity,
    pub block: Option<RenderObject>,
}

impl MyGame {
    fn new(game: &mut Game) -> Result<Self, String> {
        let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
            return Err("Filed to get scene".to_string());
        };

        let transform = Transform {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0) 
        };

        let truck = scene.spawn()
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

        Ok(Self {
            truck,
            h: 0.0,
            camera1,
            camera2,
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

            update_freefly_rotation!(self.camera1, delta_yaw, delta_pitch);
        } else if active_camera == self.camera2 {
            let delta_yaw = Rad(-mm.dx as f32 * sensitivity);
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

            set_camerafollow_position_offset!(self.camera2, new_position_offset);
            set_camerafollow_rotation_offset!(self.camera2, new_rotation_offset);
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

            set_default_camera!(camera);
        }

        if input.is_key_pressed(KeyCode::ArrowUp) {
            update_entity_position!(self.truck, Vector3::new(0.0, 0.0, speed * dt));
        }

        if input.is_key_pressed(KeyCode::ArrowDown) {
            update_entity_position!(self.truck, Vector3::new(0.0, 0.0, -speed * dt));
        }

        if input.is_key_pressed(KeyCode::KeyW) {
            update_freefly_position!(self.camera1, forward * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyS) {
            update_freefly_position!(self.camera1, -forward * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyA) {
            update_freefly_position!(self.camera1, -right * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyD) {
            update_freefly_position!(self.camera1, right * speed * dt);
        }

        if input.is_key_pressed(KeyCode::KeyE) {
            self.h += 1.0;
            let transform = Transform {
                position: vec3(0.0, 30.0, -self.h),
                rotation: Quaternion::one(),
                scale: vec3(1.0, 1.0, 1.0) 
            };

            if let Some(block) = self.block.clone() {
                let x = spawn_entity!(transform);
                add_components!(x, block);
            }
        }
    }
}

impl Gear for MyGame {
    async fn setup(&mut self, game: &GameView) {
        if let Some(block) = load_obj_model!("truck/semi.obj", game) {
            self.block = Some(RenderObject {
                lods: vec![block]
            });
        };

        let Some(truck_lod0) = load_obj_model!("truck/semi.obj", game) else {
            return;
        };

        let Some(truck_lod1) = load_obj_model!("truck/semi_lod1.obj", game) else {
            return;
        };

        let Some(truck_lod2) = load_obj_model!("truck/semi_lod2.obj", game) else {
            return;
        };

        add_components!(self.truck, RenderObject {lods: vec![truck_lod0, truck_lod1, truck_lod2]});

        // miku //

        let Some(miku_lod0) = load_obj_model!("miku/miku.obj", game) else {
            return;
        };

        let Some(miku_lod1) = load_obj_model!("miku/miku_lod1.obj", game) else {
            return;
        };

        let Some(miku_lod2) = load_obj_model!("miku/miku_lod2.obj", game) else {
            return;
        };

        const SPACE_BETWEEN: f32 = 15.0;
        const NUM_INSTANCES_PER_ROW: usize = 64;

        let positions = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
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

            spawn_entity!(miku.clone(), transform);
        }

        // random scene //

        let transform = Transform {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0)
        };

        if let Some(scene1) = load_obj_model!("scene1/scene1.obj", game) {
            spawn_entity!(RenderObject{ lods: vec![scene1] }, transform);
        };

        let transform = Transform { 
            position: vec3(40.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(0.5, 0.5, 0.5)
        };

        if let Some(scene2) = load_obj_model!("scene2/scene2.obj", game) {
            spawn_entity!( RenderObject{ lods: vec![scene2] }, transform);
        };

        let transform = Transform { 
            position: vec3(0.0, 0.0, 40.0),
            rotation: Quaternion::one(),
            scale: vec3(7.0, 7.0, 7.0)
        };

        if let Some(scene3) = load_obj_model!("scene3/scene3.obj", game) {
            spawn_entity!( RenderObject{ lods: vec![scene3] }, transform);
        };

        let transform = Transform { 
            position: vec3(25.0, 0.0, 20.0),
            rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Rad(std::f32::consts::PI)),
            scale: vec3(0.3, 0.3, 0.3)
        };

        if let Some(scene4) = load_obj_model!("scene4/scene4.obj", game) {
            spawn_entity!( RenderObject{ lods: vec![scene4] }, transform);
        };

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

        let Some(landscape_model3d) = add_model3d!("landscape", landscape_mesh) else {
            log!(LogKind::Error, "failed to make model3d for landscape");
            return;
        };

        let entity = spawn_entity!(RenderObject{lods: vec![landscape_model3d]}, landscape, Transform::identity());

        log!(LogKind::Info, "landscape loaded as {:?}", entity);
    }

    async fn update(&mut self, game: GameView) {
        self.mouse_motion(&game).await;
        self.keyboard_input(&game).await;
    }
}

pub fn main() {
    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::default());
        if let Ok(mygame) = MyGame::new(game) {
            game.add_gear("mygame".into(), mygame);
        }
    }).setup(|game| {
        let Ok(mut scene) = game.components.get_mut::<WorldScene>() else {
            return;
        };
        scene.add_gui(EngineStats::new());
        scene.add_gui(TerminalGui::new(default_terminal_commands()));
    }).run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_access_is_safe() {
        let mut game = Game::new()
            .setup(|game| {
                if let Ok(mygame) = MyGame::new(game) {
                    game.add_gear("mygame".into(), mygame);
                }
            }).setup(|game| {
                if let Err(err) = game.components.with::<WorldScene, _>(|scene| {
                    scene.add_gui(EngineStats::new());
                }) {
                    log!(LogKind::Error, "Failed to add EngineStats to scene: {}", err);
                }
            });

        for _ in 0..10 {
            game.dispatch_event(GearEvent::Update);
        }
    }
}
