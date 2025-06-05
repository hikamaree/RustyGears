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

/// Command to set the active (default) camera in the scene.
///
/// When executed, the camera with the specified `id` will be set as the
/// active camera for rendering and other view-related operations.
///
/// # Fields
/// - `id`: The identifier of the camera to be set as the active one.
#[derive(Debug)]
pub struct SetDefaultCamera {
    pub camera: Entity,
}

impl Command for SetDefaultCamera {
    fn apply(self: Box<Self>, game: &mut Game) {
        game.scene().set_active_camera(self.camera);
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
#[derive(Debug)]
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
#[derive(Debug)]
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

/// Command to move an instance by a relative offset.
///
/// This command updates the translation of the instance by adding `delta` to it.
#[derive(Debug)]
pub struct MoveInstance {
    pub entity: Entity,
    pub delta: cgmath::Vector3<f32>,
}

impl Command for MoveInstance {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Some(instance) = game.scene().world.get_mut::<Transform>(self.entity) {
            instance.position += self.delta;
        }
    }
}

/// Command to rotate an instance by a relative quaternion rotation.
///
/// This command multiplies the current rotation by the `delta` rotation.
#[derive(Debug)]
pub struct RotateInstance {
    pub entity: Entity,
    pub delta: cgmath::Quaternion<f32>,
}

impl Command for RotateInstance {
    fn apply(self: Box<Self>, game: &mut Game) {
        if let Some(instance) = game.scene().world.get_mut::<Transform>(self.entity) {
            instance.rotation = self.delta * instance.rotation;
        }
    }
}
