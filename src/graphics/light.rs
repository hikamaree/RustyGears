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

/// A uniform representing a single light in world space.
///
/// This structure is intended to be uploaded to the GPU as a uniform buffer
/// and should be tightly packed. Padding fields are included to ensure proper
/// 16-byte alignment required by `wgpu` and most GPU drivers.
///
/// # Layout (std140-compatible):
/// - `position`: The world-space position of the light (vec3)  
/// - `_padding`: Padding to align `position` to 16 bytes  
/// - `color`: The RGB color of the light (vec3)  
/// - `_padding2`: Padding to align `color` to 16 bytes
///
/// # Fields
/// - `position`: 3D position of the light in world coordinates.
/// - `_padding`: Required padding to satisfy 16-byte alignment for uniform buffers.
/// - `color`: RGB color of the light.
/// - `_padding2`: Required padding after `color` to ensure alignment.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}
