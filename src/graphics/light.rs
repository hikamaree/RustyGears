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

use crate::Transform;
use cgmath::InnerSpace;

/// Maximum number of supported lights in a single frame.
pub(crate) const MAX_LIGHTS: usize = 64;

/// Common interface for all light types.
///
/// Each light must be able to convert itself into a GPU-friendly
/// representation (`GpuLight`) for use in shaders.
pub trait Light {
    /// Converts the light into a `GpuLight` using its world transform.
    fn to_gpu(&self, transform: &Transform) -> GpuLight;
}

/// GPU-compatible enumeration of light types.
///
/// Matches shader definitions for identifying the light behavior.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GpuLightType {
    /// Omnidirectional point light that emits from a single position.
    Point = 0,
    /// Directional light that simulates sunlight or distant sources.
    Directional = 1,
    /// Spotlight with a cone-shaped area of effect.
    Spot = 2,
}

/// GPU-side representation of a light source.
///
/// All light types (point, directional, spot) share this packed layout.
/// Sent directly to shaders as part of a uniform buffer.
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
        spot_angles: [0.0; 2],
        _padding: [0; 2],
    };
}

/// A point light emitting in all directions from a single position.
#[derive(Clone, Copy)]
pub struct PointLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

/// A directional light that emits uniformly along a direction.
#[derive(Clone, Copy)]
pub struct DirectionalLight {
    pub color: [f32; 3],
    pub intensity: f32,
}

/// A spotlight emitting within a cone defined by inner and outer angles.
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
            spot_angles: [self.inner_angle.cos(), self.outer_angle.cos()],
            _padding: [0; 2],
        }
    }
}
