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
    pub j: f32,
    pub k: f32,
    pub l: f32,
    pub camera1: Entity,
    pub camera2: Entity,
}

impl MyGame {
    fn new(game: &mut Game) -> Self {
        let transform = Transform {
            position: vec3(8.0, -10.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0) 
        };

        let truck = match game.spawn_model("truck/semi", transform) {
            Ok(truck) => truck,
            Err(_) => todo!(),
        };

        let camera1 = game.scene()
            .spawn()
            .with(Camera::new())
            .with(Transform::identity())
            .with(CameraControl {
                handler: Box::new(FreeFlyCamera::new()),
            })
            .build();

        game.scene().set_active_camera(camera1);

        let camera2 = game.scene()
            .spawn()
            .with(Camera::new())
            .with(CameraControl {
                handler: Box::new(CameraFollow{
                    target: truck,
                    position_offset: vec3(0.0, 20.0, -60.0),
                    rotation_offset: Quaternion::one()
                })
            }).build();

        Self {
            sender: None,
            truck, 
            h: 0.0,
            j: 0.0,
            k: 0.0,
            l: 0.0,
            camera1,
            camera2
        }
    }
}

impl Gear for MyGame {
    fn setup(&mut self, game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);

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

                if let Err(e) = game.spawn_model("miku/miku", transform) {
                    eprintln!("{}", e);
                }
            }
        }
    }

    fn mouse_motion(&mut self, dx: f64, dy: f64, game: GameView) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let Some(active_camera) = game.scene.active_camera else {
            return;
        };

        let sensitivity = 0.005;

        let cmd = if active_camera == self.camera1 {
            let sensitivity = 0.002;
            let delta_yaw = Rad(-dx as f32 * sensitivity);
            let delta_pitch = Rad(-dy as f32 * sensitivity);

            ModifyCameraHandle::for_type::<FreeFlyCamera>(
                self.camera1,
                move |camera, _| {
                    camera.update_rotation(delta_yaw, delta_pitch);
                },
            )
        } else if active_camera == self.camera2 {
            let delta_yaw = Rad(-dx as f32 * sensitivity);

            ModifyCameraHandle::for_type::<CameraFollow>(
                self.camera2,
                move |camera, game| {
                    let Some(target_transform) = game.scene().world.get::<Transform>(camera.target) else {
                        return;
                    };

                    let rot = Quaternion::from_angle_y(delta_yaw);
                    camera.position_offset = rot.rotate_vector(camera.position_offset);

                    let camera_position = target_transform.position + camera.position_offset;

                    let mut flat_forward = target_transform.position - camera_position;
                    flat_forward.y = 0.0;

                    if flat_forward.magnitude2() < 0.0001 {
                        return;
                    }

                    let flat_forward = flat_forward.normalize();
                    let look_rotation = Quaternion::from_angle_y(
                        -Rad(flat_forward.z.atan2(flat_forward.x) + std::f32::consts::FRAC_PI_2),
                    );

                    camera.rotation_offset = look_rotation;
                }
            )
        } else {
            return;
        };

        if let Err(e) = sender.send(Box::new(cmd)) {
            eprintln!("Failed to send ModifyCameraHandle: {}", e);
        }
    }

    fn keyboard_input(&mut self, key: KeyCode, state: ElementState, game: GameView) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let dt = game.time.delta_time();

        let speed = 50.0;

        let cmd: Box<dyn Command> = match key {
            KeyCode::KeyC => {
                if state == ElementState::Released {
                    return;
                }
                let Some(active_camera) = game.scene.active_camera else {
                    return;
                };

                let camera = if active_camera == self.camera1 {
                    self.camera2
                } else {
                    self.camera1
                };

                Box::new(SetDefaultCamera { camera })
            }

            KeyCode::ArrowUp => {
                let delta = 50.0 * dt;

                Box::new(MoveInstance {
                    entity: self.truck,
                    delta: Vector3::new(0.0, 0.0, delta),
                })
            }

            KeyCode::ArrowDown => {
                let delta = 50.0 * dt;

                Box::new(MoveInstance {
                    entity: self.truck,
                    delta: Vector3::new(0.0, 0.0, -delta),
                })
            }

            KeyCode::KeyW => {
                Box::new(ModifyCameraHandle::for_type::<FreeFlyCamera>(
                        self.camera1,
                        move |camera, _| {
                            camera.update_position(camera.forward() * speed * dt);
                        },
                ))
            }
            KeyCode::KeyS => {
                Box::new(ModifyCameraHandle::for_type::<FreeFlyCamera>(
                        self.camera1,
                        move |camera, _| {
                            camera.update_position(-camera.forward() * speed * dt);
                        },
                ))
            }
            KeyCode::KeyA => {
                Box::new(ModifyCameraHandle::for_type::<FreeFlyCamera>(
                        self.camera1,
                        move |camera, _| {
                            camera.update_position(-camera.right() * speed * dt);
                        },
                ))
            }
            KeyCode::KeyD => {
                Box::new(ModifyCameraHandle::for_type::<FreeFlyCamera>(
                        self.camera1,
                        move |camera, _| {
                            camera.update_position(camera.right() * speed * dt);
                        },
                ))
            }

            KeyCode::KeyH => {
                self.h += 1.0;
                let transform = Transform {
                    position: vec3(0.0, 30.0, -self.h),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                Box::new(SpawnModel { 
                    file_path: "block/block.obj".to_string(),
                    transform,
                })
            }

            KeyCode::KeyJ => {
                self.j += 1.0;
                let transform = Transform {
                    position: vec3(-self.j, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                Box::new(SpawnModel { 
                    file_path: "ball/ball.obj".to_string(),
                    transform,
                })
            }

            KeyCode::KeyK => {
                self.k += 1.0;
                let transform = Transform {
                    position: vec3(self.k, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                Box::new(SpawnModel { 
                    file_path: "ball/ball.obj".to_string(),
                    transform,
                })
            }

            KeyCode::KeyL => {
                self.l += 1.0;
                let transform = Transform {
                    position: vec3(0.0, 30.0, self.l),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                Box::new(SpawnModel { 
                    file_path: "block/block.obj".to_string(),
                    transform,
                })
            }

            _ => {
                return;
            },
        };

        if let Err(e) = sender.send(cmd) {
            eprintln!("Failed to send MoveInstance command: {}", e);
        }
    }
}

pub fn main() {
    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::default());
        let mygame = MyGame::new(game);
        game.add_gear("mygame".into(), mygame);
    }).setup(|game| {
        game.scene().add_gui(EngineStats::new());

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
                game.scene().add_gui(EngineStats::new());

                let transform = Transform { 
                    position: vec3(0.0, 0.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0)
                };

                if let Err(e) = game.spawn_model("scene1/scene1", transform) {
                    eprintln!("{}", e);
                }
            });

        for _ in 0..10 {
            game.dispatch_event(GearEvent::MouseMotion(0.0, 0.0));
            game.dispatch_event(GearEvent::Update());
        }
    }
}
