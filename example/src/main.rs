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

use rusty_gears::math::Rad;
use rusty_gears::math::InnerSpace;
use rusty_gears::math::One;
use rusty_gears::math::Point3;
use rusty_gears::math::Rotation3;
use rusty_gears::math::Quaternion;
use rusty_gears::math::Zero;
use rusty_gears::math::Deg;
use rusty_gears::math::vec3;
use rusty_gears::math::Vector3;
use rusty_gears::*;

#[derive(Debug, Default)]
struct MyGame {
    pub sender: Option<Sender<Box<dyn Command>>>,
    kamioni: Vec<usize>,
    pub h: f32,
    pub j: f32,
    pub k: f32,
    pub l: f32,
}

impl MyGame {
    fn switch_camera(&self, game: &GameView) {
        let index = game.scene.active_camera_id().expect("no camera found");
        let id = (index) % game.scene.camera_count() + 1;
        let cmd = SetDefaultCamera { id };
        println!("Switching camera: {} -> {}", index, id);
        if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
            eprintln!("Failed to send SetDefaultCamera command: {}", e);
        }
    }
}

impl Gear for MyGame {
    fn setup(&mut self, game: &mut Game, sender: Sender<Box<dyn Command>>) {
        self.sender = Some(sender);

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

                self.kamioni.push(game.spawn_model("truck/semi.obj", transform, vec![RenderTag::PBR]));
            }
        }
    }

    fn mouse_motion(&mut self, dx: f64, dy: f64, game: GameView) {
        let camera = game.scene.active_camera().unwrap();
        let id = camera.get_id();
        let dt = game.time.delta_time();

        let yaw = camera.yaw + Rad(dx as f32 * camera.sensitivity * dt); 
        let pich = camera.pitch - Rad(dy as f32 * camera.sensitivity * dt); 

        if yaw != camera.yaw || pich != camera.pitch {
            let cmd = SetCameraRotation { id, yaw, pich, roll: Rad(0.0) };
            if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
                eprintln!("Failed to send SetCameraRotation command: {}", e);
            }
        }
    }

    fn keyboard_input(&mut self, key: KeyCode, state: ElementState, game: GameView) {
        let camera = game.scene.active_camera().unwrap();
        let id = camera.get_id();
        let dt = game.time.delta_time();

        let mut position = camera.position;

        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                position += camera.forward * camera.speed * dt;
            }

            KeyCode::KeyS | KeyCode::ArrowDown => {
                position -= camera.forward * camera.speed * dt;
            }

            KeyCode::KeyA | KeyCode::ArrowLeft => {
                position -= camera.right * camera.speed * dt;
            }

            KeyCode::KeyD | KeyCode::ArrowRight => {
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
                    render_tags: vec![RenderTag::PBR] 
                };

                if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
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
                    render_tags: vec![RenderTag::PBR] 
                };

                if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
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
                    render_tags: vec![RenderTag::PBR] 
                };

                if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
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
                    render_tags: vec![RenderTag::PBR] 
                };

                if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
                    eprintln!("Failed to send SpawnModel command: {}", e);
                }

                self.l += 1.0;
            }

            _ => {},
        }

        if position != camera.position {
            let cmd = SetCameraPosition { id, position };
            if let Err(e) = self.sender.as_ref().unwrap().send(Box::new(cmd)) {
                eprintln!("Failed to send SetCameraPosition command: {}", e);
            }
        }
    }
}

pub fn main() {
    let mut camera1 = Camera::new();
    camera1.set_position(Point3::new(0.0, 10.0, 0.0));

    let mut camera2 = Camera::new();
    camera2.set_position(Point3::new(0.0, 0.0, 0.0));

    Game::new().setup(|game| {
        game.add_gear("render".into(), Render::default());
        game.add_gear("mygame".into(), MyGame::default());
        game.scene.add_camera(camera1);
        game.scene.add_camera(camera2);
    }).setup(|game| {
        game.scene.add_gui(EngineStats::new());
    }).run();
}
