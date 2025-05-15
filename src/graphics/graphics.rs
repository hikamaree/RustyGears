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

use crate::Camera;
use crate::DEFAULT_CAMERA_BUFFER_SIZE;
use crate::LightUniform;
use crate::BufferStrategy;
use crate::Buffer;
use crate::graphics::pipeline::create_render_pipeline;
use crate::Command;
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

#[allow(dead_code)]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl Vertex for InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}





#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindGroupLayoutKey {
    Camera,
    Texture,
    Light,
    Custom(&'static str),
}

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

    pub bind_group_layouts: HashMap<BindGroupLayoutKey, Arc<wgpu::BindGroupLayout>>,
    pub pipelines: HashMap<RenderTag, wgpu::RenderPipeline>,

    pub buffers: HashMap<String, Buffer>,
    pub bind_groups: HashMap<BindGroupLayoutKey, wgpu::BindGroup>,


    pub t_count: u32,
}

impl Graphics {
    pub(crate) async fn new(window: Arc<Window>) -> Graphics {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
        .await
            .unwrap();
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
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
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

        graphics
    }

    pub fn get_pipeline(&self, tag: &RenderTag) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(tag)
    }

    pub fn create_bind_grouproup_layout(&mut self, key: BindGroupLayoutKey, entries: &[wgpu::BindGroupLayoutEntry], label: Option<&str>) {
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries,
        });
        self.bind_group_layouts.insert(key, Arc::new(layout));
    }


    pub fn create_buffer(&mut self, name: &str, size: usize, usage: wgpu::BufferUsages, strategy: BufferStrategy) {
        let buffer = Buffer::new(&self.device, size, usage, strategy, name);
        self.buffers.insert(name.to_string(), buffer);
    }

    pub fn create_bind_group(&mut self, layout_key: BindGroupLayoutKey, entries: &[wgpu::BindGroupEntry], label: Option<&str>) -> Result<(), String> {
        let layout = self.bind_group_layouts.get(&layout_key)
            .ok_or_else(|| format!("Bind group layout {:?} not found", layout_key))?;

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
            entries,
        });

        self.bind_groups.insert(layout_key, bind_group);
        Ok(())
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
            ],
            Some("texture_bind_group_layout"),
            );

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

            let render_pipeline_layout = self.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Default Render Pipeline Layout"),
                    bind_group_layouts: &[
                        self.bind_group_layouts.get(&BindGroupLayoutKey::Texture).expect("Texture bind group layout not found"),
                        self.bind_group_layouts.get(&BindGroupLayoutKey::Camera).expect("Camera bind group layout not found"),
                        self.bind_group_layouts.get(&BindGroupLayoutKey::Light).expect("Ligth bind group layout not found"),
                    ],
                    push_constant_ranges: &[],
                });

            let pbr_pipeline = {
                let shader = wgpu::ShaderModuleDescriptor {
                    label: Some("Default Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                };

                create_render_pipeline(
                    &self.device,
                    &render_pipeline_layout,
                    self.config.format,
                    Some(Texture::DEPTH_FORMAT),
                    &[ ModelVertex::desc(), InstanceRaw::desc() ],
                    shader,
                )
            };

            self.pipelines.insert(RenderTag::PBR, pbr_pipeline);

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
                color: [1.0, 1.0, 1.0],
                _padding2: 0,
            };

            self.buffers.get_mut("light").unwrap().write(&self.queue, bytemuck::cast_slice(&[light_uniform]));

            let mut buffers = std::mem::take(&mut self.buffers);

            self.create_bind_group(
                BindGroupLayoutKey::Camera,
                &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.get_mut("camera").unwrap().current().as_entire_binding(),
                }],
                Some("camera_bind_group"),
            ).unwrap();

            self.create_bind_group(
                BindGroupLayoutKey::Light,
                &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.get_mut("light").unwrap().current().as_entire_binding(),
                }],
                Some("light_bind_group"),
            ).expect("Failed to create light bind group");

            self.buffers = buffers;
    }

    pub(crate) fn update(&mut self, camera: &Camera) {
        self.queue.write_buffer(
            &self.buffers.get("camera").expect("no camera buffer found").current(),
            0,
            &camera.get_uniform(),
        );
    }


    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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

pub struct SetTrianglesCount {
    pub count: u32,
}

impl Command for SetTrianglesCount {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        game.graphics().t_count = self.count;
    }
}
