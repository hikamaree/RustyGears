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

use cgmath::Vector3;
use cgmath::Zero;

#[derive(Debug, Clone, Copy)]
pub struct RigidBody {
    pub velocity: Vector3<f32>,
    pub forces: Vector3<f32>,
    pub mass: f32,
    pub is_static: bool,
}

impl RigidBody {
    pub fn dynamic(mass: f32) -> Self {
        Self {
            velocity: Vector3::zero(),
            forces: Vector3::zero(),
            mass,
            is_static: false,
        }
    }

    pub fn static_body() -> Self {
        Self {
            velocity: Vector3::zero(),
            forces: Vector3::zero(),
            mass: f32::INFINITY, // I don't know physics, maybe should be 0.0
            is_static: true,
        }
    }

    pub fn add_force(&mut self, force: Vector3<f32>) {
        if !self.is_static {
            self.forces += force;
        }
    }

    pub fn set_velocity(&mut self, velocity: Vector3<f32>) {
        if !self.is_static {
            self.velocity = velocity;
        }
    }

    pub fn stop(&mut self) {
        self.velocity = Vector3::zero();
        self.forces = Vector3::zero();
    }

    pub fn acceleration(&self) -> Vector3<f32> {
        if self.mass == 0.0 || self.is_static {
            Vector3::zero()
        } else {
            self.forces / self.mass
        }
    }

    pub fn clear_forces(&mut self) {
        self.forces = Vector3::zero();
    }
}
