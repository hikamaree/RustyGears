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

use crate::CommandFunction;
use crate::Transform;
use crate::World;
use crate::Entity;
use crate::WorldScene;

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
    pub target: Entity,
    pub position_offset: Vector3<f32>,
    pub rotation_offset: Quaternion<f32>,
}

impl CameraFollow {
    /// Creates a new `CameraFollow` component that follows a given `target` entity.
    ///
    /// # Parameters
    /// - `target`: The entity to follow.
    ///
    /// # Returns
    /// A `CameraFollow` instance initialized with zero positional offset and unit rotation offset.
    ///
    /// # Example
    /// ```
    /// let follow = CameraFollow::new(player_entity);
    /// ```
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            position_offset: Vector3::zero(),
            rotation_offset: Quaternion::one()
        }
    }

    /// Applies an incremental offset to the camera’s positional offset.
    ///
    /// This schedules a command that, when executed, finds the `CameraFollow`
    /// handler attached to the specified `entity` and adds `delta_offset` to its
    /// existing `position_offset`.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `CameraFollow` component will be updated.
    /// - `delta_offset`: The amount to add to the camera’s current `position_offset`.
    ///
    /// # Example
    /// ```
    /// CameraFollow::update_position_offset(camera_entity, vec3(0.0, 2.0, -5.0));
    /// ```
    pub fn update_position_offset(entity: Entity, delta_offset: Vector3<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<CameraFollow>() {
                            camera.position_offset += delta_offset;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Applies a relative rotation to the camera’s rotational offset.
    ///
    /// This command multiplies the existing `rotation_offset` by the provided `delta_offset`,
    /// effectively rotating the camera around its current orientation.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `CameraFollow` component will be updated.
    /// - `delta_offset`: The quaternion representing the incremental rotation to apply.
    ///
    /// # Example
    /// ```
    /// let delta = Quaternion::from_angle_y(Rad(0.1));
    /// CameraFollow::update_rotation_offset(camera_entity, delta);
    /// ```
    pub fn update_rotation_offset(entity: Entity, delta_offset: Quaternion<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<CameraFollow>() {
                            camera.rotation_offset = delta_offset * camera.rotation_offset;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Sets the absolute rotational offset of the camera relative to its target.
    ///
    /// Replaces any previously applied rotation offset.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `CameraFollow` component will be updated.
    /// - `rotation`: The new absolute rotation offset as a quaternion.
    ///
    /// # Example
    /// ```
    /// let rotation = Quaternion::from_angle_x(Rad(-0.3));
    /// CameraFollow::set_rotation_offset(camera_entity, rotation);
    /// ```
    pub fn set_rotation_offset(entity: Entity, rotation: Quaternion<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<CameraFollow>() {
                            camera.rotation_offset = rotation;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Sets the absolute positional offset between the camera and its target.
    ///
    /// Replaces the current `position_offset` value.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `CameraFollow` component will be updated.
    /// - `position`: The new position offset relative to the followed entity.
    ///
    /// # Example
    /// ```
    /// CameraFollow::set_position_offset(camera_entity, vec3(0.0, 5.0, -10.0));
    /// ```
    pub fn set_position_offset(entity: Entity, position: Vector3<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<CameraFollow>() {
                            camera.position_offset = position;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Changes which entity the camera follows.
    ///
    /// Replaces the `target` entity of the `CameraFollow` component.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `CameraFollow` component will be updated.
    /// - `target`: The new target entity to follow.
    ///
    /// # Example
    /// ```
    /// CameraFollow::set_target_entity(camera_entity, player_entity);
    /// ```
    pub fn set_target_entity(entity: Entity, target: Entity) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<CameraFollow>() {
                            camera.target = target;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
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
/// Suitable for first-person, spectator, or editor-style cameras that can move
/// freely in 3D space without a fixed target.
#[derive(Clone)]
pub struct FreeFlyCamera {
    pub position: Vector3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl FreeFlyCamera {
    /// Creates a new `FreeFlyCamera` with default position and zero rotation.
    ///
    /// # Returns
    /// A `FreeFlyCamera` instance at the origin `(0, 0, 0)` facing along the negative Z-axis.
    ///
    /// # Example
    /// ```
    /// let camera = FreeFlyCamera::new();
    /// ```
    pub fn new() -> Self {
        Self {
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            position: Vector3::zero(),
        }
    }

    /// Clamps the pitch angle to prevent flipping when looking too far up or down.
    ///
    /// The pitch is limited to ±(π/2 - 0.01) radians to avoid gimbal lock.
    ///
    /// # Example
    /// ```
    /// camera.lock_pitch();
    /// ```
    pub fn lock_pitch(&mut self) {
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        self.pitch.0 = self.pitch.0.clamp(-limit, limit);
    }

    /// Computes the normalized forward direction vector from the current yaw and pitch.
    ///
    /// The result represents the direction the camera is facing in world space.
    ///
    /// # Returns
    /// A unit `Vector3<f32>` pointing forward from the camera’s perspective.
    ///
    /// # Example
    /// ```
    /// let forward = camera.forward();
    /// ```
    pub fn forward(&self) -> Vector3<f32> {
        Vector3 {
            x: -self.yaw.sin() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: -self.yaw.cos() * self.pitch.cos(),
        }.normalize()
    }

    /// Computes the normalized right vector perpendicular to the camera’s forward vector.
    ///
    /// This can be used for strafing movement along the X-axis of the camera’s local space.
    ///
    /// # Returns
    /// A unit `Vector3<f32>` pointing right from the camera’s perspective.
    ///
    /// # Example
    /// ```
    /// let right = camera.right();
    /// ```
    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(Vector3::unit_y()).normalize()
    }

    /// Builds a `Transform` component that represents the camera’s world-space position and orientation.
    ///
    /// The transform uses a unit scale and combines yaw (Y-axis) and pitch (X-axis) rotations.
    ///
    /// # Returns
    /// A `Transform` describing the camera’s world-space transform.
    ///
    /// # Example
    /// ```
    /// let transform = camera.build_transform();
    /// ```
    pub fn build_transform(&self) -> Transform {
        let rotation = Quaternion::from_angle_y(self.yaw) * Quaternion::from_angle_x(self.pitch);
        Transform {
            position: self.position,
            rotation,
            scale: vec3(1.0, 1.0, 1.0),
        }
    }

    /// Applies a relative translation to the camera’s position.
    ///
    /// This schedules a command that adds the given `delta` vector to the current camera position.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `FreeFlyCamera` component will be modified.
    /// - `delta`: The position delta (movement vector) to apply.
    ///
    /// # Example
    /// ```
    /// FreeFlyCamera::update_position(camera_entity, vec3(0.0, 0.0, -1.0));
    /// ```
    pub fn update_position(entity: Entity, delta: Vector3<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<FreeFlyCamera>() {
                            camera.position += delta;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Applies incremental yaw and pitch adjustments to the camera’s rotation.
    ///
    /// This schedules a command that adds `delta_yaw` and `delta_pitch` to the current rotation angles,
    /// and then clamps the pitch to prevent flipping.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `FreeFlyCamera` component will be modified.
    /// - `delta_yaw`: Change in yaw (rotation around the Y-axis).
    /// - `delta_pitch`: Change in pitch (rotation around the X-axis).
    ///
    /// # Example
    /// ```
    /// FreeFlyCamera::update_rotation(camera_entity, Rad(0.05), Rad(-0.02));
    /// ```
    pub fn update_rotation(entity: Entity, delta_yaw: Rad<f32>, delta_pitch: Rad<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<FreeFlyCamera>() {
                            camera.yaw += delta_yaw;
                            camera.pitch += delta_pitch;
                            camera.lock_pitch();
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Sets the absolute position of the `FreeFlyCamera`.
    ///
    /// Replaces the existing position with the provided `position` vector.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `FreeFlyCamera` component will be modified.
    /// - `position`: The new position in world space.
    ///
    /// # Example
    /// ```
    /// FreeFlyCamera::set_position(camera_entity, vec3(10.0, 5.0, -3.0));
    /// ```
    pub fn set_position(entity: Entity, position: Vector3<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<FreeFlyCamera>() {
                            camera.position = position;
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
    }

    /// Sets the absolute rotation of the camera in yaw and pitch.
    ///
    /// Replaces any previously accumulated rotation and clamps the pitch afterward.
    ///
    /// # Parameters
    /// - `entity`: The camera entity whose `FreeFlyCamera` component will be modified.
    /// - `yaw`: The new yaw angle (horizontal rotation).
    /// - `pitch`: The new pitch angle (vertical rotation).
    ///
    /// # Example
    /// ```
    /// FreeFlyCamera::set_rotation(camera_entity, Rad(1.57), Rad(-0.3));
    /// ```
    pub fn set_rotation(entity: Entity, yaw: Rad<f32>, pitch: Rad<f32>) {
        let cmd = CommandFunction {
            run: Box::new(move |game| {
                if let Ok(mut scene) = game.components.get_mut::<WorldScene>() {
                    if let Some(control) = scene.world.get_mut::<CameraControl>(entity) {
                        if let Some(camera) = control.handler.as_any_mut().downcast_mut::<FreeFlyCamera>() {
                            camera.yaw = yaw;
                            camera.pitch = pitch;
                            camera.lock_pitch();
                        }
                    }
                }
            }),
        };
        crate::send_command(cmd);
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
