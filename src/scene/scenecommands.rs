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

use crate::Entity;
use crate::Transform;
use crate::Command;
use crate::Game;
use crate::Camera;

use cgmath::Point3;

/// Command to set the active (default) camera in the scene.
///
/// When executed, the camera with the specified `id` will be set as the
/// active camera for rendering and other view-related operations.
///
/// # Fields
/// - `id`: The identifier of the camera to be set as the active one.
pub struct SetDefaultCamera {
    pub camera: Entity,
}

impl Command for SetDefaultCamera {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.scene().set_active_camera(self.camera);
    }
}

/// Command to add a new camera to the scene.
///
/// The camera is initialized with the given position and rotation
/// expressed through yaw, pitch, and roll values (in radians).
///
/// # Fields
/// - `position`: The position of the camera in 3D space.
/// - `yaw`: The horizontal rotation of the camera (around the Y-axis).
/// - `pitch`: The vertical rotation of the camera (around the X-axis).
/// - `roll`: The tilt rotation of the camera (around the Z-axis).
pub struct AddCamera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Command for AddCamera {
    fn apply(self: Box<Self>, game: &mut Game) {
        let mut camera = Camera::new();
        camera.set_position(self.position);
        camera.set_rotation(cgmath::Rad(self.yaw), cgmath::Rad(self.pitch), cgmath::Rad(self.roll));
        game.scene().add_camera(camera);
    }
}

/// Command to spawn a 3D model into the scene.
///
/// The model is loaded from the specified `file_path`, transformed by
/// the provided `transform`, and associated with the given `render_tags`
/// for rendering purposes.
///
/// # Fields
/// - `file_path`: The path to the model file (e.g., `.obj`, `.gltf`).
/// - `transform`: The transformation applied to the model when spawning.
/// - `render_tags`: A list of render tags defining how the object will be rendered.
pub struct SpawnModel {
    pub file_path: String,
    pub transform: Transform,
}

impl Command for SpawnModel {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Err(e) = game.spawn_model(&self.file_path, self.transform) {
            println!("{}", e);
        }
    }
}

/// Command to set the transformation of an instance in the scene.
///
/// This command updates the transform of an existing instance by its `id`
/// with the provided `transform`.
///
/// # Fields
/// - `id`: The identifier of the instance whose transformation will be updated.
/// - `transform`: The new transformation to be applied to the instance.
pub struct SetInstanceTransform {
    pub entity: Entity,
    pub transform: Transform,
}

impl Command for SetInstanceTransform {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Some(instance) = game.scene().world.get_mut::<Transform>(self.entity) {
            *instance = self.transform;
        }
    }
}
