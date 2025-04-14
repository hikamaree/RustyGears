use crate::graphics::pipeline::create_render_pipeline;
use crate::Vertex;
use crate::ModelVertex;
use std::collections::HashMap;
use crate::{Projection, RenderTag};
use crate::Texture;
use std::sync::Arc;
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

pub struct Graphics {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    pub depth_texture: Texture,
    pub projection: Projection,

    pub bind_group_layouts: HashMap<BindGroupLayoutKey, Arc<wgpu::BindGroupLayout>>,
    pub pipelines: HashMap<RenderTag, wgpu::RenderPipeline>,
}

impl Graphics {
    pub(crate) async fn new(window: Arc<Window>) -> Graphics {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            projection,
            bind_group_layouts,
            pipelines,
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

    pub fn create_render_pipeline(
        &mut self,
        render_tag: RenderTag,
        layout: &wgpu::PipelineLayout,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader_path: &str,
    ) {
        let shader_src = std::fs::read_to_string(shader_path)
            .unwrap_or_else(|_| panic!("Failed to load shader at {}", shader_path));

        let shader_label = format!("{:?} Shader", render_tag);
        
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: Some(&shader_label),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        };

        let pipeline = create_render_pipeline(
            &self.device,
            layout,
            self.config.format,
            Some(Texture::DEPTH_FORMAT),
            vertex_layouts,
            shader_desc,
        );

        self.pipelines.insert(render_tag, pipeline);
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

            let render_pipeline = {
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

            self.pipelines.insert(RenderTag::PBR, render_pipeline);
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.projection.resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
}
