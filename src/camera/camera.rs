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

use crate::Projection;
use crate::Transform;
use cgmath::vec3;
use cgmath::vec4;
use cgmath::EuclideanSpace;
use cgmath::InnerSpace;
use cgmath::Matrix;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::SquareMatrix;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Zero;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// The `Camera` struct represents a camera in a 3D space.

#[derive(Clone)]
pub struct Camera {
    pub id: u64,
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    pub speed: f32,
    pub sensitivity: f32,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    frustum: [cgmath::Vector4<f32>; 6],
}

impl Camera {
    /// Creates a new camera with an initial position, yaw, and pitch values.
    ///
    /// # Arguments
    /// * `position` - A tuple containing three values representing the camera's position in 3D space.
    /// * `yaw` - The yaw angle for rotating the camera around the Y axis.
    /// * `pitch` - The pitch angle for rotating the camera around the X axis.
    ///
    /// # Returns
    /// Returns an instance of the camera.
    pub fn new() -> Self {
        Camera {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            view_position: [0.0; 4],
            view_proj: Matrix4::identity().into(),
            speed: 40.0,
            sensitivity: 0.4,
            forward: vec3(0.0, 0.0, -1.0),
            right: Vector3::zero(),
            frustum: [vec4(0.0, 0.0, 0.0, 0.0); 6],
        }
    }

    /// Returns the camera's ID.
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Calculates the camera's view matrix based on the current yaw and pitch values.
    ///
    /// # Returns
    /// A view matrix that determines how objects will be rendered in relation to the camera.
    pub fn calc_matrix(&self, transform: &Transform) -> Matrix4<f32> {
        let forward = transform.forward();
        Matrix4::look_to_rh(
            Point3::from_vec(transform.position),
            forward,
            Vector3::unit_y(),
        )
    }

    /// Updates the camera's view frustum planes from the current view-projection matrix.
    ///
    /// This method extracts six clipping planes (left, right, bottom, top, near, far) from
    /// the combined view-projection matrix and normalizes them. These planes are used for
    /// frustum culling, allowing efficient visibility checks against objects in the scene.
    pub fn update_frustum(&mut self) {
        let m = cgmath::Matrix4::from(self.view_proj);

        let planes = [
            m.row(3) + m.row(0),
            m.row(3) - m.row(0),
            m.row(3) + m.row(1),
            m.row(3) - m.row(1),
            m.row(3) + m.row(2),
            m.row(3) - m.row(2),
        ];

        self.frustum = planes.map(|v| {
            let normal = v.truncate();
            let length = normal.magnitude();
            v / length
        });
    }

    /// Checks if a bounding sphere is inside or intersects the camera's view frustum.
    ///
    /// # Parameters
    /// - `center`: The center of the sphere in world space.
    /// - `radius`: The radius of the sphere.
    ///
    /// # Returns
    /// - `true` if the sphere is at least partially inside the view frustum.
    /// - `false` if the sphere is completely outside and can be culled.
    pub fn can_see(&self, center: Vector3<f32>, radius: f32) -> bool {
        self.frustum.iter().all(|plane| {
            plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w >= -radius
        })
    }

    /// Checks if a bounding sphere is inside or intersects the camera's view frustum using homogeneous coordinates.
    ///
    /// This method is similar to [`can_see`] but operates on a 4D position vector (e.g., when working in clip space or
    /// with pre-transformed coordinates). Each plane is treated as a 4D vector, and visibility is determined using
    /// a dot product between the plane and the sphere center in homogeneous space.
    ///
    /// # Parameters
    /// - `center`: The center of the bounding sphere as a homogeneous 4D vector (`x, y, z, 1.0`).
    /// - `radius`: The radius of the bounding sphere.
    ///
    /// # Returns
    /// - `true` if the sphere is at least partially inside the view frustum.
    /// - `false` if the sphere is completely outside the frustum and can be culled.
    pub fn can_see4(&self, center: Vector4<f32>, radius: f32) -> bool {
        self.frustum
            .iter()
            .all(|plane| plane.dot(center) >= -radius)
    }

    /// Updates the camera's view and projection matrix.
    ///
    /// # Arguments
    /// * `projection` - The projection used to update the projection matrix.
    pub fn update_view_proj(&mut self, transform: &Transform, projection: &Projection) {
        self.view_position = Vector4::new(
            transform.position.x,
            transform.position.y,
            transform.position.z,
            1.0,
        )
        .into();
        let view = self.calc_matrix(transform);
        self.view_proj = (projection.calc_matrix() * view).into();
        self.update_frustum();
    }

    /// Returns the camera's uniform containing view and projection data.
    ///
    /// # Arguments
    /// * `light_count` - The number of active lights to include in the uniform.
    ///
    /// # Returns
    /// A uniform that contains the camera's position, matrices, and light count as a byte slice.
    /// Must be 112 bytes (28 floats) due to WGSL struct alignment requirements.
    pub fn get_uniform(&self, light_count: u32) -> Box<[u8]> {
        bytemuck::cast_slice(&[
            self.view_position[0],
            self.view_position[1],
            self.view_position[2],
            self.view_position[3],
            self.view_proj[0][0],
            self.view_proj[0][1],
            self.view_proj[0][2],
            self.view_proj[0][3],
            self.view_proj[1][0],
            self.view_proj[1][1],
            self.view_proj[1][2],
            self.view_proj[1][3],
            self.view_proj[2][0],
            self.view_proj[2][1],
            self.view_proj[2][2],
            self.view_proj[2][3],
            self.view_proj[3][0],
            self.view_proj[3][1],
            self.view_proj[3][2],
            self.view_proj[3][3],
            light_count as f32,
            0.0f32,
            0.0f32,
            0.0f32, // padding for vec4 alignment
            0.0f32,
            0.0f32,
            0.0f32,
            0.0f32, // Extra padding to match WGSL struct size (112 bytes)
        ])
        .into()
    }
}
