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

use cgmath::InnerSpace;
use crate::Transform;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

pub trait Light {
    fn to_gpu(&self, transform: &Transform) -> GpuLight;
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GpuLightType {
    Point = 0,
    Directional = 1,
    Spot = 2,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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

#[derive(Clone, Copy)]
pub struct PointLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub struct DirectionalLight {
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Clone, Copy)]
pub struct SpotLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Light for PointLight {
    fn to_gpu(&self, transform: &Transform) -> GpuLight {
        GpuLight {
            position: transform.position.into(),
            radius: self.radius,
            color: self.color,
            intensity: self.intensity,
            direction: [0.0; 3],
            light_type: GpuLightType::Point as u32,
            spot_angles: [0.0, 0.0],
            _padding: [0; 2],
        }
    }
}

impl Light for DirectionalLight {
    fn to_gpu(&self, transform: &Transform) -> GpuLight {
        GpuLight {
            position: [0.0; 3],
            radius: 0.0,
            color: self.color,
            intensity: self.intensity,
            direction: (-transform.forward()).normalize().into(),
            light_type: GpuLightType::Directional as u32,
            spot_angles: [0.0, 0.0],
            _padding: [0; 2],
        }
    }
}

impl Light for SpotLight {
    fn to_gpu(&self, transform: &Transform) -> GpuLight {
        GpuLight {
            position: transform.position.into(),
            radius: self.radius,
            color: self.color,
            intensity: self.intensity,
            direction: (-transform.forward()).normalize().into(),
            light_type: GpuLightType::Spot as u32,
            spot_angles: [
                self.inner_angle.cos(),
                self.outer_angle.cos(),
            ],
            _padding: [0; 2],
        }
    }
}
