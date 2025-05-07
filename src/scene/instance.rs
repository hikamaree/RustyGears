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

use crate::InstanceRaw;

/// A struct representing a 3D transformation that combines position, rotation, and scale.
///
/// The `Transform` struct is used to represent an object's position, rotation, and scale in 3D space.
/// This transformation can be applied to objects in the scene to position them, rotate them, or scale them.
///
/// # Fields
/// - `position`: A 3D vector representing the position of the object in the scene.
/// - `rotation`: A quaternion representing the object's rotation in space.
/// - `scale`: A 3D vector representing the scale factors along the X, Y, and Z axes.
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>
}

impl Transform {
    /// Converts the `Transform` to an `InstanceRaw` for raw rendering data.
    ///
    /// This method prepares the instance's transformation (position, rotation, scale)
    /// in a format that can be used for rendering in the graphics pipeline.
    ///
    /// # Returns
    /// An `InstanceRaw` struct containing the instance's model and normal matrix.
    pub fn raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation)
                * cgmath::Matrix4::from_nonuniform_scale(
                    self.scale.x,
                    self.scale.y,
                    self.scale.z
                )).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

/// An enum representing different rendering tags for objects in the scene.
///
/// Render tags are used to classify and specify how an object should be rendered.
/// For example, objects may be rendered with different materials or effects, like PBR (Physically Based Rendering),
/// Unlit, Wireframe mode, etc. Custom tags can also be created with a string value.
///
/// # Variants
/// - `PBR`: Indicates the object should be rendered with physically-based rendering materials.
/// - `Unlit`: Indicates the object should be rendered without lighting (e.g., for UI elements).
/// - `Wireframe`: Indicates the object should be rendered in wireframe mode (lines only).
/// - `ShadowMap`: A tag for objects used in shadow mapping for lighting purposes.
/// - `Custom(String)`: Allows for a custom render tag identified by a string value.
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum RenderTag {
    PBR,
    Unlit,
    Wireframe,
    ShadowMap,
    Custom(String),
}

/// A struct representing an instance of a 3D object in the scene.
///
/// An instance is a specific object in the game world that can have a transformation (position, rotation, scale),
/// render tags for how it should be rendered, and a unique ID. Multiple instances of the same model can exist in the scene,
/// each with its own transformation and render tags.
///
/// # Fields
/// - `id`: A unique identifier for the instance.
/// - `transform`: The transformation (position, rotation, scale) of the instance in the scene.
/// - `render_tags`: A list of render tags that specify how the instance should be rendered (e.g., PBR, Unlit, etc.).
/// - `name`: The name of the instance, typically used for identification in the scene.
#[derive(Debug, Clone)]
pub struct Instance {
    id: usize,
    pub transform: Transform,
    pub raw: InstanceRaw,
    pub render_tags: Vec<RenderTag>,
    pub name: String,
}

impl Instance {
    /// Creates a new instance with the specified name, transformation, and render tags.
    ///
    /// This function generates a unique ID for the instance and adds it to the scene.
    ///
    /// # Parameters
    /// - `name`: The name of the instance.
    /// - `transform`: The transformation (position, rotation, scale) for the instance.
    /// - `render_tags`: The render tags specifying how the instance should be rendered.
    ///
    /// # Returns
    /// A new `Instance` with the specified parameters.
    pub fn new(name: String, transform: Transform, render_tags: Vec<RenderTag>) -> Self {
        Self {
            id: Instance::gen_id(),
            transform,
            raw: transform.raw(),
            render_tags,
            name,
        }
    }

    /// Returns the unique identifier for this instance.
    ///
    /// # Returns
    /// The unique ID of the instance.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Generates a unique ID for each instance.
    ///
    /// This method uses a static counter to ensure each instance gets a unique ID.
    ///
    /// # Returns
    /// A unique identifier for the instance.
    fn gen_id() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
