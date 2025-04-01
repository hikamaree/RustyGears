use crate::Texture;
use crate::window::RenderData;
use std::sync::Mutex;
use crate::Camera;
use cgmath::Rotation3;
use std::sync::Arc;
use winit::window::Window;

pub(crate) struct Graphics {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
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


        let state = Graphics {
            window,
            surface,
            device,
            queue,
            config,
        };

        state
    }

    pub(crate) fn resize(&mut self, scene: &mut RenderData, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            scene.projection.resize(new_size.width, new_size.height);
            scene.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            scene.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub(crate) fn update(&mut self, scene: &mut RenderData, camera: Arc<Mutex<Camera>>) {
        camera.lock().unwrap().update_view_proj(&scene.projection);
        self.queue.write_buffer(
            &scene.camera_buffer,
            0,
            &camera.lock().unwrap().get_uniform(),
        );

        let old_position: cgmath::Vector3<_> = scene.light_uniform.position.into();
        scene.light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0)) * old_position).into();
        self.queue.write_buffer(
            &scene.light_buffer,
            0,
            bytemuck::cast_slice(&[scene.light_uniform]),
        );
    }
}
