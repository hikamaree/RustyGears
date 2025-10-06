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

use crate::InstanceRaw;
use crate::Camera;
use crate::DEFAULT_CAMERA_BUFFER_SIZE;
use crate::LightUniform;
use crate::BufferStrategy;
use crate::Buffer;
use crate::graphics::pipeline::*;
use crate::Projection;
use crate::Vertex;
use crate::ModelVertex;
use std::collections::HashMap;
use crate::RenderTag;
use crate::Texture;
use std::sync::Arc;
use wgpu::Adapter;
use wgpu::Instance;
use winit::window::Window;

/// A central object responsible for GPU resource management, pipeline setup, and surface presentation.
///
/// The `Graphics` struct encapsulates the core state required to render frames using WGPU. It owns the
/// logical device, command queue, rendering surface, and all configurations associated with the window output.
/// In addition to the raw WGPU components, it stores texture resources, camera projection settings,
/// bind group layouts, shader pipelines, GPU buffers, and runtime bind groups.
///
/// Most high-level rendering systems interact with this type to access WGPU internals safely and consistently.
#[derive(Clone)]
pub struct Graphics {
    pub window: Arc<Window>,
    pub instance: Instance,
    pub adapter: Adapter,
    pub surface: Arc<wgpu::Surface<'static>>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    pub depth_texture: Texture,
    pub projection: Projection,

    pub bind_group_layouts: HashMap<BindGroupLayoutKey, wgpu::BindGroupLayout>,
    pub pipelines: HashMap<RenderTag, wgpu::RenderPipeline>,

    pub buffers: HashMap<String, Buffer>,
    pub bind_groups: HashMap<BindGroupLayoutKey, wgpu::BindGroup>,

    pub t_count: u32,
}

impl Graphics {
    pub(crate) fn new(window: Arc<Window>) -> Result<Graphics, String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .map_err(|e| format!("Failed to create surface: {e}"))?;

        let (adapter, device, queue) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                .await
                    .ok_or_else(|| "Failed to find a suitable GPU adapter.".to_string())?;

                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::default(),
                            memory_hints: Default::default(),
                        },
                        None,
                    )
                    .await
                    .map_err(|e| format!("Failed to create device: {e}"))?;

                Ok::<_, String>((adapter, device, queue))
            })
        })?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .or_else(|| surface_caps.formats.get(0).copied())
            .ok_or_else(|| "No surface formats available.".to_string())?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 1000.0);

        let bind_group_layouts = HashMap::new();
        let pipelines = HashMap::new();

        let mut graphics = Graphics {
            window,
            instance,
            adapter,
            surface: Arc::new(surface),
            device,
            queue,
            config,
            size,
            depth_texture,
            projection,
            bind_group_layouts,
            pipelines,
            buffers: HashMap::new(),
            bind_groups: HashMap::new(),
            t_count: 0,
        };

        graphics.initialize_default_resources();

        graphics.set_resolution(graphics.window.inner_size());

        Ok(graphics)
    }

    pub fn get_pipeline(&self, tag: &RenderTag) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(tag)
    }

    pub fn create_bind_grouproup_layout(&mut self, key: BindGroupLayoutKey, entries: &[wgpu::BindGroupLayoutEntry], label: Option<&str>) {
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries,
        });
        self.bind_group_layouts.insert(key, layout);
    }


    pub fn create_buffer(&mut self, name: &str, size: usize, usage: wgpu::BufferUsages, strategy: BufferStrategy) {
        let buffer = Buffer::new(&self.device, size, usage, strategy, name);
        self.buffers.insert(name.to_string(), buffer);
    }

    pub fn create_bind_group(&mut self, layout_key: BindGroupLayoutKey, entries: &[wgpu::BindGroupEntry], label: Option<&str>) {
        if let Some(layout) = self.bind_group_layouts.get(&layout_key) {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label,
                layout,
                entries,
            });
            self.bind_groups.insert(layout_key, bind_group);
        }
    }

    fn initialize_default_resources(&mut self) {
        self.create_bind_grouproup_layout(
            BindGroupLayoutKey::Texture,
            &[
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
            }],
            Some("texture_bind_group_layout"));

        self.create_bind_grouproup_layout(
            BindGroupLayoutKey::Camera,
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            Some("camera_bind_group_layout"),
        );

        self.create_bind_grouproup_layout(
            BindGroupLayoutKey::Light,
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            Some("light_bind_group_layout"),
        );

        if let (Some(texture_layout), Some(camera_layout), Some(light_layout)) = (
            self.bind_group_layouts.get(&BindGroupLayoutKey::Texture),
            self.bind_group_layouts.get(&BindGroupLayoutKey::Camera),
            self.bind_group_layouts.get(&BindGroupLayoutKey::Light),
        ) {
            let render_pipeline_layout = self.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Default Render Pipeline Layout"),
                    bind_group_layouts: &[
                        texture_layout,
                        camera_layout,
                        light_layout,
                    ],
                    push_constant_ranges: &[],
                });

            let shader_descriptor = wgpu::ShaderModuleDescriptor {
                label: Some("Default Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };

            let opaque_pipeline = create_opaque_pipeline(
                &self.device,
                &render_pipeline_layout,
                self.config.format,
                Texture::DEPTH_FORMAT,
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader_descriptor.clone(),
            );
            self.pipelines.insert(RenderTag::Opaque, opaque_pipeline);

            let sorted_pipeline = create_sorted_transparent_pipeline(
                &self.device,
                &render_pipeline_layout,
                self.config.format,
                Texture::DEPTH_FORMAT,
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader_descriptor.clone(),
            );
            self.pipelines.insert(RenderTag::SortedTransparent, sorted_pipeline);

            let weighted_pipeline = create_weighted_transparent_pipeline(
                &self.device,
                &render_pipeline_layout,
                Texture::ACCUM_FORMAT,
                Texture::REVEALAGE_FORMAT,
                Texture::DEPTH_FORMAT,
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader_descriptor,
            );
            self.pipelines.insert(RenderTag::WeightedTransparent, weighted_pipeline);
        }

        self.create_buffer(
            "camera",
            DEFAULT_CAMERA_BUFFER_SIZE,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            BufferStrategy::Single,
        );

        self.create_buffer(
            "light",
            std::mem::size_of::<LightUniform>(),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            BufferStrategy::Single,
        );

        let light_uniform = LightUniform {
            position: [0.0, 100.0, -20.0],
            _padding: 0,
            color: [0.0, 0.0, 0.0],
            _padding2: 0,
        };


        if let Some(light_buffer) = self.buffers.get_mut("light") {
            light_buffer.write(&self.queue, bytemuck::cast_slice(&[light_uniform]));
        }

        let buffers = std::mem::take(&mut self.buffers);

        if let Some(light_buffer) = buffers.get("light") {
            self.create_bind_group(
                BindGroupLayoutKey::Light,
                &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.current().as_entire_binding(),
                }],
                Some("light_bind_group"),
            );
        }


        if let Some(camera_buffer) = buffers.get("camera") {
            self.create_bind_group(
                BindGroupLayoutKey::Camera,
                &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.current().as_entire_binding(),
                }],
                Some("camera_bind_group"),
            );
        }


        self.buffers = buffers;
    }

    pub(crate) fn update(&mut self, camera: &Camera) {
        if let Some(camer_buffer) = self.buffers.get("camera") {
            self.queue.write_buffer(
                &camer_buffer.current(),
                0,
                &camera.get_uniform(),
            );
        }
    }

    pub(crate) fn set_resolution(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.projection.resize(new_size.width, new_size.height);
        }
    }
}

/// A unique key used to identify bind group layouts within the renderer.
///
/// This enum is used to map specific purposes (such as camera uniforms, textures, or lighting data)
/// to corresponding `wgpu::BindGroupLayout` entries. The `Custom` variant allows extending the system
/// with user-defined layout identifiers that remain globally consistent across shaders and pipeline creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindGroupLayoutKey {
    Camera,
    Texture,
    Light,
    Custom(&'static str),
}

pub struct SetResolution {
    pub resolution: winit::dpi::PhysicalSize<u32>,
}

impl crate::Command for SetResolution {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        if let Ok(mut graphics) = game.components.get_mut::<Graphics>() {
            graphics.set_resolution(self.resolution);
        }
    }
}

#[macro_export]
macro_rules! set_resolution {
    ( $x:expr, $y:expr ) => {{
        $crate::send_command($crate::SetResolution {
            resolution: winit::dpi::PhysicalSize::new($x, $y),
        });
    }};
}

