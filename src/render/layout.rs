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

use std::any::TypeId;
use std::collections::HashMap;

/// Trait for types that have a GPU bind group layout.
///
/// Each implementor defines how its data is bound to shaders.
/// Layouts are created once per device and reused across frames.
pub trait HasBindGroupLayout: 'static {
    /// Returns the unique type ID for this layout.
    fn layout_id() -> TypeId;

    /// Returns the WGPU descriptor for creating this layout.
    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static>;
}

/// Registry for GPU bind group layouts.
///
/// Manages creation and access to bind group layouts organized by type.
/// Layouts are created once at initialization and reused throughout the render loop.
#[derive(Clone)]
pub struct LayoutRegistry {
    layouts: HashMap<TypeId, wgpu::BindGroupLayout>,
}

impl LayoutRegistry {
    /// Creates a new registry and registers all built-in layouts.
    pub fn new(device: &wgpu::Device) -> Self {
        let mut registry = Self {
            layouts: HashMap::new(),
        };
        registry.register::<CameraLayout>(device);
        registry.register::<LightLayout>(device);
        registry.register::<TextureLayout>(device);
        registry.register::<ShadowTextureLayout>(device);
        registry.register::<ShadowCameraLayout>(device);
        registry
    }

    /// Registers a layout for a given type.
    pub fn register<T: HasBindGroupLayout>(&mut self, device: &wgpu::Device) {
        let layout = device.create_bind_group_layout(&T::layout_descriptor());
        self.layouts.insert(T::layout_id(), layout);
    }

    /// Gets a layout by type, panicking if not found.
    ///
    /// Use this after ensuring layouts are registered at startup.
    pub fn get<T: HasBindGroupLayout>(&self) -> &wgpu::BindGroupLayout {
        self.layouts
            .get(&T::layout_id())
            .expect("Bind group layout not registered. Ensure LayoutRegistry::new() was called.")
    }

    /// Gets a layout by type, returning None if not found.
    pub fn get_opt<T: HasBindGroupLayout>(&self) -> Option<&wgpu::BindGroupLayout> {
        self.layouts.get(&T::layout_id())
    }
}

// Built-in layout implementations

const CAMERA_ENTRIES: [wgpu::BindGroupLayoutEntry; 1] = [wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
}];

pub struct CameraLayout;

impl HasBindGroupLayout for CameraLayout {
    fn layout_id() -> TypeId {
        TypeId::of::<CameraLayout>()
    }

    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &CAMERA_ENTRIES,
        }
    }
}

const LIGHT_ENTRIES: [wgpu::BindGroupLayoutEntry; 1] = [wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
}];

pub struct LightLayout;

impl HasBindGroupLayout for LightLayout {
    fn layout_id() -> TypeId {
        TypeId::of::<LightLayout>()
    }

    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("light_bind_group_layout"),
            entries: &LIGHT_ENTRIES,
        }
    }
}

const TEXTURE_ENTRIES: [wgpu::BindGroupLayoutEntry; 6] = [
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 4,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 5,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    },
];

pub struct TextureLayout;

impl HasBindGroupLayout for TextureLayout {
    fn layout_id() -> TypeId {
        TypeId::of::<TextureLayout>()
    }

    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &TEXTURE_ENTRIES,
        }
    }
}

const SHADOW_TEXTURE_ENTRIES: [wgpu::BindGroupLayoutEntry; 2] = [
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
        count: None,
    },
];

pub struct ShadowTextureLayout;

impl HasBindGroupLayout for ShadowTextureLayout {
    fn layout_id() -> TypeId {
        TypeId::of::<ShadowTextureLayout>()
    }

    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_texture_bind_group_layout"),
            entries: &SHADOW_TEXTURE_ENTRIES,
        }
    }
}

const SHADOW_CAMERA_ENTRIES: [wgpu::BindGroupLayoutEntry; 1] = [wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::VERTEX,
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
}];

pub struct ShadowCameraLayout;

impl HasBindGroupLayout for ShadowCameraLayout {
    fn layout_id() -> TypeId {
        TypeId::of::<ShadowCameraLayout>()
    }

    fn layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_camera_bind_group_layout"),
            entries: &SHADOW_CAMERA_ENTRIES,
        }
    }
}
