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
use std::sync::OnceLock;
use wgpu::Queue;
use wgpu::Device;
use image::GenericImageView;

/// A GPU-resident texture with associated view and sampler.
///
/// This struct represents a complete bindable texture unit in WGPU. It includes the underlying
/// `wgpu::Texture` resource used for storing texel data, a `wgpu::TextureView` that provides access
/// to the texture from shaders, and a `wgpu::Sampler` that defines how the texture is filtered and sampled.
///
/// `Texture` instances are typically used for material inputs such as diffuse, normal, and dissolve maps,
/// but they can also be used as render targets or for procedurally generated data.
///
/// `Texture` objects are used for material textures (diffuse, normal, dissolve), 
/// render targets, and procedural texture creation.
#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

/// A lazily initialized global fallback texture used when actual textures are missing.
///
/// This dummy texture is a solid white pixel.
static DEFAULT_TEXTURE: OnceLock<Arc<Texture>> = OnceLock::new();

impl Texture {
    /// Recommended texture format used for weighted blended OIT accumulation buffer.
    pub const ACCUM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

    /// Format used for the revealage buffer in weighted blended OIT.
    pub const REVEALAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    /// Format used for depth textures in rendering pipelines.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture that matches the given surface configuration.
    ///
    /// This is typically used as a depth buffer in render passes.
    ///
    /// # Arguments
    /// * `device` - The GPU device.
    /// * `config` - The surface config (used to get width/height).
    /// * `label` - A debug label for the texture.
    ///
    /// # Returns
    /// A depth `Texture` object with `DEPTH_FORMAT`.
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[Self::DEPTH_FORMAT],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Loads a texture from a byte buffer (e.g., from a file or memory).
    ///
    /// # Arguments
    /// * `device` - The GPU device.
    /// * `queue` - The GPU queue to upload the texture data.
    /// * `bytes` - The image data in memory.
    /// * `label` - Optional debug name.
    /// * `is_normal_map` - Whether the texture is a normal map (affects gamma).
    ///
    /// # Returns
    /// A `Texture` if loading succeeded, or an error message.
    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: &str, is_normal_map: bool) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|err| format!("Failed to decode image: {}", err))?;
        Ok(Self::from_image(device, queue, &img, Some(label), is_normal_map))
    }

    /// Creates a texture from a decoded image.
    ///
    /// # Arguments
    /// * `device` - The GPU device.
    /// * `queue` - The GPU queue to upload data.
    /// * `img` - The image to convert into a GPU texture.
    /// * `label` - Optional debug label.
    /// * `is_normal_map` - Whether to use linear or sRGB format.
    ///
    /// # Returns
    /// A fully constructed `Texture` ready for use.
    pub fn from_image(device: &Device, queue: &Queue, img: &image::DynamicImage, label: Option<&str>, is_normal_map: bool) -> Self {
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Creates a 1x1 solid-color texture (RGBA).
    ///
    /// Useful for dummy materials or procedural effects.
    ///
    /// # Arguments
    /// * `device` - The GPU device.
    /// * `queue` - The command queue for texture upload.
    /// * `color` - The RGBA color as f32 values (0.0–1.0).
    /// * `label` - Optional debug label.
    /// * `is_normal_map` - Whether the texture is a normal map (affects format).
    ///
    /// # Returns
    /// A simple solid-color `Texture`.
    pub fn from_color(device: &Device, queue: &Queue, color: [f32; 4], label: Option<&str>, is_normal_map: bool) -> Self {
        let rgba = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Returns a lazily initialized fallback texture used when no actual texture is provided.
    ///
    /// Creates a 1x1 solid gray pixel texture ([0.5, 0.5, 0.5, 1.0]) and uploads it to the GPU.
    /// This ensures shaders always have a valid texture bound, preventing rendering errors.
    ///
    /// The texture is created only once and shared across the application using an `Arc<Texture>`.
    ///
    /// # Arguments
    /// * `device` - GPU device used for allocation.
    /// * `queue` - GPU queue used for data upload.
    ///
    /// # Returns
    /// A shared [`Arc<Texture>`] fallback texture.
    pub fn default(device: &wgpu::Device, queue: &wgpu::Queue) -> Arc<Texture> {
        DEFAULT_TEXTURE.get_or_init(|| {
            Arc::new(Texture::from_color(
                    device,
                    queue,
                    [0.5, 0.5, 0.5, 1.0],
                    Some("dummy_fallback"),
                    false,
            ))
        }).clone()
    }
}
