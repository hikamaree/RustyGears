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

use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Arc;

use crate::Texture;

static DEFAULT_MATERIAL: OnceCell<Arc<Material>> = OnceCell::new();

#[derive(Debug, Clone)]
pub enum TextureSource {
    File(PathBuf),
    Color([f32; 4]),
}

#[derive(Debug, Clone)]
pub struct MaterialData {
    pub name: String,
    pub diffuse: Option<TextureSource>,
    pub normal: Option<TextureSource>,
    pub dissolve: Option<(TextureSource, f32)>,
    pub dissolve_only: f32,
    pub use_weighted: bool,
}

impl MaterialData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            diffuse: None,
            normal: None,
            dissolve: None,
            dissolve_only: 1.0,
            use_weighted: false,
        }
    }

    pub fn with_diffuse(mut self, source: Option<TextureSource>) -> Self {
        self.diffuse = source;
        self
    }

    pub fn with_normal(mut self, source: Option<TextureSource>) -> Self {
        self.normal = source;
        self
    }

    pub fn with_dissolve(mut self, source: Option<TextureSource>, dissolve: f32) -> Self {
        self.dissolve = source.map(|s| (s, dissolve));
        self.dissolve_only = dissolve;
        self
    }

    pub fn use_weighted_blended(&self) -> bool {
        self.use_weighted
    }

    pub fn with_weighted_blended(mut self, use_weighted: bool) -> Self {
        self.use_weighted = use_weighted;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub data: Arc<MaterialData>,
    pub diffuse_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub dissolve_texture: Option<Arc<Texture>>,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        data: MaterialData,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        diffuse_texture: Option<Arc<Texture>>,
        normal_texture: Option<Arc<Texture>>,
        dissolve_texture: Option<Arc<Texture>>,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let dummy = Texture::default(device, queue);

        let mut entries = Vec::new();

        if let Some(ref diffuse_texture) = diffuse_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        if let Some(ref normal_texture) = normal_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        if let Some(ref dissolve_texture) = dissolve_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&dissolve_texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&dissolve_texture.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&dummy.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&dummy.sampler),
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: Some(&data.name),
        });

        Self {
            data: Arc::new(data),
            diffuse_texture,
            normal_texture,
            dissolve_texture,
            bind_group,
        }
    }

    pub fn has_transparency_texture(&self) -> bool {
        self.dissolve_texture.is_some()
    }

    pub fn use_weighted_blended(&self) -> bool {
        self.data.use_weighted_blended()
    }

    pub fn dissolve(&self) -> f32 {
        self.data.dissolve_only
    }

    pub fn default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Arc<Material> {
        DEFAULT_MATERIAL
            .get_or_init(|| {
                let material = Material::new(
                    MaterialData::new(String::from("default")),
                    device,
                    queue,
                    None,
                    None,
                    None,
                    layout,
                );

                Arc::new(material)
            })
            .clone()
    }
}
