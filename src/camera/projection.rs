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

use cgmath::Rad;
use cgmath::Matrix4;
use cgmath::perspective;

/// Represents a perspective projection used to transform 3D coordinates into clip space.
///
/// This struct encapsulates the parameters for a standard perspective projection matrix,
/// including field of view, aspect ratio, and near/far clipping planes. It's typically used
/// when rendering 3D scenes from a camera's point of view.
#[derive(Clone, Copy)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    /// Creates a new perspective projection with the given screen dimensions and parameters.
    ///
    /// # Parameters
    /// - `width`: The width of the render surface (in pixels).
    /// - `height`: The height of the render surface (in pixels).
    /// - `fovy`: The vertical field of view (in radians). Can be passed as a `Rad<f32>` or raw `f32`.
    /// - `znear`: The distance to the near clipping plane. Must be > 0.
    /// - `zfar`: The distance to the far clipping plane. Must be > `znear`.
    ///
    /// # Returns
    /// A new `Projection` instance with computed aspect ratio and stored parameters.
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    /// Updates the internal aspect ratio based on new surface dimensions.
    ///
    /// This should be called whenever the window or render target is resized,
    /// to ensure the projection matrix remains correct.
    ///
    /// # Parameters
    /// - `width`: New width of the render surface.
    /// - `height`: New height of the render surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculates the perspective projection matrix based on the stored parameters.
    ///
    /// The resulting matrix transforms camera-space coordinates into clip space,
    /// accounting for field of view, aspect ratio, and near/far clipping planes.
    ///
    /// # Returns
    /// A 4x4 perspective projection matrix (`Matrix4<f32>`) suitable for GPU uniform upload.
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
