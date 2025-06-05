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

use crate::Model;
use crate::InstanceRaw;

use std::ops::Neg;

use cgmath::InnerSpace;
use cgmath::Quaternion;
use cgmath::Vector3;
use cgmath::One;
use cgmath::Zero;

#[derive(Debug, Clone)]
pub struct ModelInstance {
    pub name: String,
}

/// Represents a renderable object in the scene, which may have multiple
/// levels of detail (LODs) depending on camera distance or rendering strategy.
///
/// This structure holds a list of `Model` instances, where each `Model`
/// corresponds to a specific LOD. LOD index 0 is the highest quality,
/// and higher indices represent lower detail versions of the model.
///
/// LOD selection is typically based on distance from the camera,
/// allowing the engine to improve performance by reducing geometric complexity
/// for far-away objects.
///
/// # Fields
/// - `lods`: A list of models sorted by LOD level, from high to low quality.
///           Each `Model` contains mesh data and GPU-ready geometry.
#[derive(Clone)]
pub struct RenderObject {
    pub lods: Vec<Model>,
}

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

    /// Returns the forward direction vector derived from the current rotation.
    ///
    /// This is typically used to determine the direction the object is facing.
    ///
    /// # Returns
    /// A normalized vector pointing in the forward (negative Z) direction.
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation * Vector3::unit_z().neg().normalize()
    }

    /// Returns the right direction vector derived from the current rotation.
    ///
    /// This is typically used to determine the local right direction of the object.
    ///
    /// # Returns
    /// A normalized vector pointing to the right (positive X) direction.
    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(Vector3::unit_y()).normalize()
    }


    /// Returns a default identity transform.
    ///
    /// The identity transform represents no translation, no rotation, and uniform scale (1.0, 1.0, 1.0).
    ///
    /// # Returns
    /// A `Transform` positioned at the origin, with no rotation, and a scale of 1 on all axes.
    pub fn identity() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
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
/// - `Opaque`: The default render tag for fully opaque geometry. Rendered first without sorting.
/// - `SortedTransparent`: Transparent objects that require depth-based sorting for correct rendering (e.g. glass).
/// - `WeightedTransparent`: Transparent objects rendered using weighted blended order-independent transparency.
/// - `Custom(String)`: A custom render tag identified by a string, useful for user-defined rendering passes or effects.
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum RenderTag {
    Opaque,
    SortedTransparent,
    WeightedTransparent,
    Custom(String),
}
