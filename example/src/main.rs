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

use rusty_gears::math::Rotation3;
use rusty_gears::math::Vector3;
use rusty_gears::math::Rad;
use rusty_gears::math::One;
use rusty_gears::math::Quaternion;
use rusty_gears::math::vec3;
use rusty_gears::*;

#[derive(Debug, Default)]
struct MyGame {
    pub sender: Option<Sender<Box<dyn Command>>>,
    pub truck: Option<Entity>,
    pub h: f32,
    pub j: f32,
    pub k: f32,
    pub l: f32,
    pub camera1: Option<Entity>,
}

impl MyGame {
    fn switch_camera(&self, game: &GameView) {
        let next_cam: Option<Entity>;
        if game.scene.active_camera == self.camera1 {
            next_cam = self.truck;
        } else {
            next_cam = self.camera1;
        }

        let Some(camera) = next_cam else {
            return;
        };

        let cmd = SetDefaultCamera { camera };
        
        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        if let Err(e) = sender.send(Box::new(cmd)) {
            eprintln!("Failed to send SetDefaultCamera command: {}", e);
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

        let transform = Transform {
            position: vec3(8.0, -10.0, 0.0),
            rotation: Quaternion::one(),
            scale: vec3(1.0, 1.0, 1.0) 
        };

        self.truck = match game.spawn_model("truck/semi", transform) {
            Ok(truck) => Some(truck),
            Err(_) => None,
        };


        let camera1 = Camera::new();
        self.camera1 = Some(game.scene().add_camera(camera1));

        let mut camera2 = Camera::new();
        camera2.set_rotation(Rad(3.14 / 2.0), Rad(0.0), Rad(0.0));
        camera2.set_position((8.0, 5.0, -20.0).into());

        if let Some(truck) = self.truck {
            game.scene().world.insert(truck, camera2);
        }
    }

    fn mouse_motion(&mut self, dx: f64, dy: f64, game: GameView) {
        let Some(camera1) = self.camera1 else {
            return;
        };

        let Some(camera) = game.scene.get_camera(camera1) else {
            return;
        };

        let dt = game.time.delta_time();

        let yaw = camera.yaw + Rad(dx as f32 * camera.sensitivity * dt); 
        let pitch = camera.pitch - Rad(dy as f32 * camera.sensitivity * dt); 

        if yaw != camera.yaw || pitch != camera.pitch {
            let cmd = SetCameraRotation { entity: camera1, yaw, pitch, roll: Rad(0.0) };

            let Some(sender) = self.sender.as_ref() else {
                return;
            };

            if let Err(e) = sender.send(Box::new(cmd)) {
                eprintln!("Failed to send SetCameraRotation command: {}", e);
            }
        }
    }

    fn keyboard_input(&mut self, key: KeyCode, state: ElementState, game: GameView) {
        let Some(camera1) = self.camera1 else {
            return;
        };

        let Some(camera) = game.scene.get_camera(camera1) else {
            return;
        };

        let Some(truck) = self.truck else {
            return;
        };

        let Some(truck_camera) = game.scene.get_camera(truck) else {
            return;
        };

        let Some(sender) = self.sender.as_ref() else {
            return;
        };

        let dt = game.time.delta_time();

        let mut position = camera.position;

        match key {
            KeyCode::ArrowUp => {
                let delta = 50.0 * dt;

                let Some(transform) = game.scene.world.get::<Transform>(truck) else {
                    return;
                };

                let mut transform = transform.clone();

                transform.position.z += delta;

                let cmd = SetInstanceTransform { 
                    entity: truck,
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }

                let mut position = truck_camera.clone().position;
                position.z += delta;

                let cmd = SetCameraPosition {
                    entity: truck,
                    position
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }
            }

            KeyCode::ArrowDown => {
                let delta = 50.0 * dt;
                let Some(transform) = game.scene.world.get::<Transform>(truck) else {
                    return;
                };

                let mut transform = transform.clone();

                transform.position.z -= delta;

                let cmd = SetInstanceTransform { 
                    entity: truck,
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }


                let mut position = truck_camera.clone().position;
                position.z -= delta;

                let cmd = SetCameraPosition {
                    entity: truck,
                    position
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }
            }

            KeyCode::KeyW => {
                position += camera.forward * camera.speed * dt;
            }

            KeyCode::KeyS => {
                position -= camera.forward * camera.speed * dt;
            }

            KeyCode::KeyA => {
                position -= camera.right * camera.speed * dt;
            }

            KeyCode::KeyD => {
                position += camera.right * camera.speed * dt;
            }

            KeyCode::KeyC => if state == ElementState::Pressed {
                self.switch_camera(&game);
            }

            KeyCode::KeyH =>  if state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(0.0, 30.0, -self.h),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                let cmd = SpawnModel { 
                    file_path: "block/block.obj".to_string(),
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }
                self.h += 1.0;
            }

            KeyCode::KeyJ => if state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(-self.j, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                let cmd = SpawnModel { 
                    file_path: "ball/ball.obj".to_string(),
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }

                self.j += 1.0;
            }

            KeyCode::KeyK => if state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(self.k, 30.0, 0.0),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                let cmd = SpawnModel { 
                    file_path: "ball/ball.obj".to_string(),
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }

                self.k += 1.0;
            }

            KeyCode::KeyL => if state == ElementState::Pressed {
                let transform = Transform {
                    position: vec3(0.0, 30.0, self.l),
                    rotation: Quaternion::one(),
                    scale: vec3(1.0, 1.0, 1.0) 
                };

                let cmd = SpawnModel { 
                    file_path: "block/block.obj".to_string(),
                    transform,
                };

                if let Err(e) = sender.send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }

                self.l += 1.0;
            }

            _ => {},
        }

        if position != camera.position {
            let cmd = SetCameraPosition { entity: camera1, position };
            if let Err(e) = sender.send(Box::new(cmd)) {
                eprintln!("Failed to send SetCameraPosition command: {}", e);
            }
        }
    }
}

pub fn main() {
    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::default());
        game.add_gear("mygame".into(), MyGame::default());
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
                game.add_gear("mygame".into(), MyGame::default());

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
