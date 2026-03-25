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

pub(crate) const MAX_LIGHTS: usize = 64;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GpuLightType {
    Point = 0,
    Directional = 1,
    Spot = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLight {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub direction: [f32; 3],
    pub light_type: u32,
    pub spot_angles: [f32; 2],
    pub _padding: [u32; 2],
}

impl GpuLight {
    pub const NULL: GpuLight = GpuLight {
        position: [0.0; 3],
        radius: 0.0,
        color: [0.0; 3],
        intensity: 0.0,
        direction: [0.0; 3],
        light_type: 0,
        spot_angles: [0.0, 0.0],
        _padding: [0; 2],
    };
}
