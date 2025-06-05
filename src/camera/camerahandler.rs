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

use std::any::Any;
use dyn_clone::DynClone;

use crate::Game;
use crate::Transform;
use crate::World;
use crate::Entity;
use crate::Command;

use cgmath::Vector3;
use cgmath::Rotation3;
use cgmath::Rad;
use cgmath::Quaternion;
use cgmath::InnerSpace;
use cgmath::Angle;
use cgmath::vec3;
use cgmath::One;
use cgmath::Zero;

/// Trait for dynamic, polymorphic camera behavior implementations.
///
/// Any camera behavior that needs to provide its own way of computing
/// a camera `Transform` should implement this trait. It supports downcasting
/// through `as_any` for dynamic type access.
pub trait CameraHandle: Send + Sync + DynClone {
    /// Computes the world-space transform for the camera.
    ///
    /// # Parameters
    /// - `world`: Reference to the ECS world.
    /// - `entity`: The entity associated with this camera.
    ///
    /// # Returns
    /// - A `Transform` representing the camera’s position and orientation in the world.
    fn get_camera_transform(&self, world: &World, entity: Entity) -> Transform;

    /// Returns a reference to the implementor as `Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the implementor as `Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

dyn_clone::clone_trait_object!(CameraHandle);

/// Component that associates an entity with dynamic camera behavior.
///
/// Wraps a `Box<dyn CameraHandle>` to enable polymorphic camera logic
/// such as `FreeFlyCamera` or `CameraFollow`.
#[derive(Clone)]
pub struct CameraControl {
    /// The dynamic camera behavior handler.
    pub handler: Box<dyn CameraHandle>,
}

/// Camera behavior that follows a target entity with a positional and rotational offset.
///
/// Useful for third-person or orbital camera setups.
#[derive(Clone)]
pub struct CameraFollow {
    /// The target entity the camera follows.
    pub target: Entity,

    /// Offset from the target position.
    pub position_offset: Vector3<f32>,

    /// Offset applied to the target's rotation.
    pub rotation_offset: Quaternion<f32>,
}

impl CameraFollow {
    /// Creates a new `CameraFollow` with the specified target.
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            position_offset: Vector3::zero(),
            rotation_offset: Quaternion::one()
        }
    }

    /// Sets a new target entity for the camera.
    pub fn set_target(&mut self, target: Entity) {
        self.target = target;
    }

    /// Sets the position offset relative to the target.
    pub fn set_position_offset(&mut self, offset: Vector3<f32>) {
        self.position_offset = offset;
    }

    /// Sets the rotation offset relative to the target.
    pub fn set_rotation_offset(&mut self, offset: Quaternion<f32>) {
        self.rotation_offset = offset;
    }
}

impl CameraHandle for CameraFollow {
    fn get_camera_transform(&self, world: &World, _entity: Entity) -> Transform {
        let target_transform = match world.get::<Transform>(self.target) {
            Some(t) => t.clone(),
            None => return Transform::identity(),
        };

        let rotation = target_transform.rotation * self.rotation_offset;
        let position = target_transform.position + self.position_offset;

        Transform {
            position,
            rotation,
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Camera behavior for free-flying movement controlled via yaw and pitch.
///
/// Suitable for first-person or editor-style cameras.
#[derive(Clone)]
pub struct FreeFlyCamera {
    /// Current camera position.
    pub position: Vector3<f32>,

    /// Rotation around the Y axis (horizontal).
    pub yaw: Rad<f32>,

    /// Rotation around the X axis (vertical).
    pub pitch: Rad<f32>,
}

impl FreeFlyCamera {
    /// Creates a new `FreeFlyCamera` with default position and orientation.
    pub fn new() -> Self {
        Self {
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            position: Vector3::zero(),
        }
    }

    /// Updates the camera's yaw and pitch based on delta input.
    pub fn update_rotation(&mut self, delta_yaw: Rad<f32>, delta_pitch: Rad<f32>) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;

        let limit = std::f32::consts::FRAC_PI_2 - 0.01;

        self.pitch.0 = self.pitch.0.clamp(-limit, limit);
    }

    /// Translates the camera by the given delta vector.
    pub fn update_position(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    /// Computes the forward vector based on current yaw and pitch.
    pub fn forward(&self) -> Vector3<f32> {
        Vector3 {
            x: -self.yaw.sin() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: -self.yaw.cos() * self.pitch.cos(),
        }.normalize()
    }

    /// Computes the right vector relative to the forward vector.
    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(Vector3::unit_y()).normalize()
    }

    /// Builds a `Transform` from the camera’s internal state.
    pub fn build_transform(&self) -> Transform {
        let rotation = Quaternion::from_angle_y(self.yaw) * Quaternion::from_angle_x(self.pitch);
        Transform {
            position: self.position,
            rotation,
            scale: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl CameraHandle for FreeFlyCamera {
    fn get_camera_transform(&self, _world: &World, _entity: Entity) -> Transform {
        self.build_transform()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A command that mutably modifies the camera behavior of a specific entity.
///
/// This command enables runtime mutation of dynamic camera types using downcasting.
pub struct ModifyCameraHandle {
    entity: Entity,
    f: Box<dyn FnOnce(&mut dyn CameraHandle, &mut Game) + Send + Sync>,
}

impl ModifyCameraHandle {
    /// Creates a `ModifyCameraHandle` command that only applies to cameras of a specific type `T`.
    ///
    /// The internal function will downcast the camera handler to `T` and execute the provided closure if successful.
    ///
    /// # Type Parameters
    /// - `T`: The expected concrete camera type.
    ///
    /// # Parameters
    /// - `entity`: The entity whose camera is to be modified.
    /// - `f`: A closure that receives a mutable reference to the concrete camera type.
    pub fn for_type<T: CameraHandle + 'static>(
        entity: Entity,
        f: impl FnOnce(&mut T, &mut Game) + Send + Sync + 'static,
    ) -> Self {
        Self {
            entity,
            f: Box::new(move |handle, game| {
                if let Some(concrete) = handle.as_any_mut().downcast_mut::<T>() {
                    f(concrete, game);
                } else {
                    eprintln!(
                        "ModifyCameraHandle: type mismatch (expected {})",
                        std::any::type_name::<T>()
                    );
                }
            }),
        }
    }
}

impl Command for ModifyCameraHandle {
    fn apply(self: Box<Self>, game: &mut Game) {
        let scene = unsafe { &mut *game.scene.get() };

        let Some(control) = scene.world.get_mut::<CameraControl>(self.entity) else {
            return;
        };
        (self.f)(&mut *control.handler, game);
    }
}
