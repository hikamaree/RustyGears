// SPDX-License-Identifier: GPL-3.0-or-later

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

use cgmath::Point3;
use cgmath::Rad;
use crate::Command;

/// A command that sets the position of a camera in the scene.
///
/// This command is used to move a camera to a new position specified
/// by a 3D point. The command looks up the camera by its `id` and
/// updates its position if found.
///
/// # Fields
/// - `id`: The unique identifier of the camera to be moved.
/// - `position`: The new position for the camera in world space.
pub struct SetCameraPosition {
    /// The ID of the camera to modify.
    pub id: u64,

    /// The new position to set, in world space.
    pub position: Point3<f32>,
}

impl Command for SetCameraPosition {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        if let Some(camera) = game.scene.get_camera_mut(self.id) {
            camera.set_position(self.position);
        }
    }
}

/// A command that sets the rotation of a camera in the scene.
///
/// This command is used to orient a camera using yaw, pitch, and roll angles.
/// The command looks up the camera by its `id` and updates its rotation if found.
///
/// # Fields
/// - `id`: The unique identifier of the camera to be rotated.
/// - `yaw`: Rotation around the vertical axis (Y-axis).
/// - `pich`: Rotation around the lateral axis (X-axis). (Note: likely a typo; should be `pitch`)
/// - `roll`: Rotation around the longitudinal axis (Z-axis).
pub struct SetCameraRotation {
    /// The ID of the camera to modify.
    pub id: u64,

    /// Yaw angle in radians (rotation around Y-axis).
    pub yaw: Rad<f32>,

    /// Pitch angle in radians (rotation around X-axis).
    pub pich: Rad<f32>, // ← ako je typo u nazivu polja, mogu da ti ponudim i ispravku

    /// Roll angle in radians (rotation around Z-axis).
    pub roll: Rad<f32>,
}

impl Command for SetCameraRotation {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        if let Some(camera) = game.scene.get_camera_mut(self.id) {
            camera.set_rotation(self.yaw, self.pich, self.roll);
        }
    }
}
