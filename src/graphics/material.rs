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

use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::Texture;

static DEFAULT_MATERIAL: OnceCell<Arc<Material>> = OnceCell::new();

/// A GPU-ready material that contains textures and render-time parameters.
///
/// `Material` encapsulates the data required to render a surface, including its
/// diffuse, normal, and dissolve (transparency) textures. It also stores the precomputed
/// `wgpu::BindGroup` used to bind the material to the GPU pipeline.
///
/// This is typically created per unique material defined in a model file (e.g., `.mtl`).
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub dissolve_texture: Option<Texture>,
    pub bind_group: wgpu::BindGroup,
    pub dissolve: f32,
}

impl Material {
    /// Constructs a new `Material` and its GPU bind group.
    ///
    /// Any missing texture will be substituted with a dummy texture to ensure
    /// the bind group is fully populated and valid.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device used to create resources.
    /// * `queue` - The command queue to upload texture data.
    /// * `name` - Material name, used for labeling.
    /// * `diffuse_texture` - Optional diffuse (albedo) texture.
    /// * `normal_texture` - Optional normal map.
    /// * `dissolve_texture` - Optional alpha mask for transparency.
    /// * `dissolve` - Default dissolve value (used when texture is not present).
    /// * `layout` - Bind group layout defining expected bindings.
    ///
    /// # Returns
    /// A fully constructed `Material` instance.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
        dissolve_texture: Option<Texture>,
        dissolve: f32,
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

        if let Some(ref normal_texture) = diffuse_texture {
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
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            dissolve_texture,
            dissolve,
            bind_group,
        }
    }

    /// Returns `true` if the material uses a dissolve texture (alpha mask).
    pub fn has_transparency_texture(&self) -> bool {
        self.dissolve_texture.is_some()
    }

    pub fn use_weighted_blended(&self) -> bool {
        // TODO
        false
    }

    /// Returns a lazily initialized global fallback material.
    ///
    /// This function attempts to load a material using `assets/default.png` for diffuse,
    /// and falls back to white if not available. Normal and dissolve textures are optional.
    /// The resulting material is stored globally and reused.
    ///
    /// # Arguments
    /// - `device`: The GPU device.
    /// - `queue`: The GPU queue.
    /// - `layout`: The bind group layout used for materials.
    ///
    /// # Returns
    /// `Arc<Material>` representing a global fallback material.
    pub fn default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Arc<Material> {
        DEFAULT_MATERIAL.get_or_init(|| {
            let material = Material::new(
                device,
                queue,
                "default",
                None,
                None,
                None,
                1.0,
                layout,
            );

            Arc::new(material)
        }).clone()
    }
}

