use crate::render::cull::{compute_frustum_planes, is_in_frustum, transform_bounding_sphere};
use crate::render::filter::EntityFilter;
use crate::render::gpu_resources::GpuResources;
use crate::render::pass::PassContext;
use crate::render::pass::PassData;
use crate::render::pass::PassOutput;
use crate::render::pass::RenderPass;
use crate::render::pass_id::PassId;
use crate::render::render_resources::RenderResources;
use crate::render::sort::NoSort;
use crate::render::sort::SortStrategy;
use crate::render::targets::TargetDescriptor;
use crate::render::targets::TargetSize;
use crate::Camera;
use crate::DirectionalLight;
use crate::Model3d;
use crate::RenderObject;
use crate::Transform;
use crate::Vertex;
use crate::WorldScene;
use cgmath::EuclideanSpace;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ShadowPass {
    sort: NoSort,
    pipeline: RwLock<Option<wgpu::RenderPipeline>>,
    cascade_resolution: u32,
}

impl ShadowPass {
    pub fn new() -> Self {
        Self {
            sort: NoSort,
            pipeline: RwLock::new(None),
            cascade_resolution: 2048,
        }
    }

    fn ensure_pipeline(&self, gpu: &GpuResources, resources: &RenderResources) {
        if self.pipeline.read().unwrap().is_some() {
            return;
        }

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shadow Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()),
            });

        let shadow_camera_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::CameraLayout>()
        {
            Some(l) => l,
            None => return,
        };

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[shadow_camera_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[crate::ModelVertex::desc(), crate::InstanceRaw::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 4,
                        slope_scale: 4.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        *self.pipeline.write().unwrap() = Some(pipeline);
    }

    fn calc_light_view_proj(
        &self,
        ctx: &PassContext,
        light_position: Vector3<f32>,
        light_forward: Vector3<f32>,
    ) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(
            Point3::from_vec(light_position),
            light_forward,
            Vector3::unit_y(),
        );

        let (_resolution, size, near, far) = ctx.shadow_config();
        let left = -size;
        let right = size;
        let bottom = -size;
        let top = size;

        let ortho = Matrix4::new(
            2.0 / (right - left),
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / (top - bottom),
            0.0,
            0.0,
            0.0,
            0.0,
            -2.0 / (far - near),
            0.0,
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -(far + near) / (far - near),
            1.0,
        );

        ortho * view
    }
}

impl RenderPass for ShadowPass {
    fn id(&self) -> PassId {
        PassId::SHADOW
    }

    fn name(&self) -> &'static str {
        "Shadow"
    }

    fn filter(&self) -> &'static EntityFilter {
        static FILTER: std::sync::LazyLock<EntityFilter> =
            std::sync::LazyLock::new(EntityFilter::new);
        &FILTER
    }

    fn sort(&self) -> &dyn SortStrategy {
        &self.sort
    }

    fn outputs(&self) -> Vec<PassOutput> {
        vec![PassOutput {
            id: PassId::SHADOW,
            descriptor: TargetDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                size: TargetSize::Fixed(self.cascade_resolution, self.cascade_resolution),
                sample_count: 1,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
        }]
    }

    fn inputs(&self) -> Vec<PassId> {
        vec![]
    }

    fn should_run(&self, scene: &WorldScene) -> bool {
        scene.world.query2::<Transform, DirectionalLight>().len() > 0
    }

    fn collect(
        &self,
        _scene: &WorldScene,
        _camera: &Camera,
        _camera_transform: &Transform,
    ) -> PassData {
        PassData::empty()
    }

    fn execute(
        &self,
        ctx: &mut PassContext,
        gpu: &GpuResources,
        _config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
        _data: &PassData,
    ) {
        self.ensure_pipeline(gpu, resources);

        let pipeline_guard = self.pipeline.read().unwrap();
        let pipeline = match pipeline_guard.as_ref() {
            Some(p) => p,
            None => return,
        };

        let resolution = ctx.shadow_cascade_resolution();
        let shadow_target = ctx.targets.allocate(
            &gpu.device,
            &TargetDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                size: TargetSize::Fixed(resolution, resolution),
                sample_count: 1,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
            ctx.screen_size.0,
            ctx.screen_size.1,
        );

        let shadow_view = match ctx.targets.get_view(shadow_target) {
            Some(v) => v,
            None => return,
        };

        let light_entity = ctx
            .scene
            .world
            .query2::<DirectionalLight, Transform>()
            .iter()
            .next()
            .map(|(e, _, _)| *e);

        let Some(light_entity) = light_entity else {
            return;
        };

        let light_transform = ctx.scene.get_camera_transform(light_entity);
        let light_forward = light_transform.forward();

        let light_view_proj =
            self.calc_light_view_proj(ctx, light_transform.position, light_forward);

        let frustum_planes = compute_frustum_planes(&light_view_proj);

        let light_uniform_data = [
            light_transform.position.x,
            light_transform.position.y,
            light_transform.position.z,
            1.0f32,
            light_view_proj[0][0],
            light_view_proj[0][1],
            light_view_proj[0][2],
            light_view_proj[0][3],
            light_view_proj[1][0],
            light_view_proj[1][1],
            light_view_proj[1][2],
            light_view_proj[1][3],
            light_view_proj[2][0],
            light_view_proj[2][1],
            light_view_proj[2][2],
            light_view_proj[2][3],
            light_view_proj[3][0],
            light_view_proj[3][1],
            light_view_proj[3][2],
            light_view_proj[3][3],
        ];
        let light_uniform = bytemuck::cast_slice(&light_uniform_data);

        if let Some(shadow_camera_buffer) = resources.get_buffer("shadow_camera") {
            gpu.queue
                .write_buffer(&shadow_camera_buffer.current(), 0, light_uniform);
        }

        let shadow_camera_bind_group = ctx.shadow_camera_bind_group;

        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, Some(shadow_camera_bind_group), &[]);

        let mut instances_by_model: HashMap<(Model3d, usize), Vec<crate::InstanceRaw>> =
            HashMap::new();

        for (_entity, render_obj, transform) in
            ctx.scene.world.query2::<RenderObject, Transform>().iter()
        {
            if render_obj.lods.is_empty() {
                continue;
            }

            let model_mat = Matrix4::from(transform.raw().model);

            for lod_index in 0..render_obj.lods.len() {
                let model3d = &render_obj.lods[lod_index];

                let Some(model) = ctx.scene.models3d.get(model3d) else {
                    continue;
                };

                for (mesh_index, mesh) in model.meshes.iter().enumerate() {
                    if !matches!(mesh.data.render_tag, crate::RenderTag::Opaque) {
                        continue;
                    }

                    let (world_center, world_radius) = transform_bounding_sphere(
                        mesh.data.bounding_sphere.center,
                        mesh.data.bounding_sphere.radius,
                        &model_mat,
                    );

                    if !is_in_frustum(&frustum_planes, world_center, world_radius) {
                        continue;
                    }

                    instances_by_model
                        .entry((model3d.clone(), mesh_index))
                        .or_default()
                        .push(transform.raw());
                }
            }
        }

        for ((model3d, mesh_index), instances) in instances_by_model.iter() {
            if instances.is_empty() {
                continue;
            }

            let Some(model) = ctx.scene.models3d.get(model3d) else {
                continue;
            };

            let mesh = &model.meshes[*mesh_index];

            let key = format!("shadow:{:?}:lod{}:mesh{}", model3d.path, 0, mesh_index);

            let buffer = resources.get_or_create_buffer_with(
                &gpu.queue,
                key.as_str(),
                || {
                    crate::Buffer::new(
                        &gpu.device,
                        instances.len().next_power_of_two()
                            * std::mem::size_of::<crate::InstanceRaw>(),
                        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        crate::BufferStrategy::Triple,
                        &key,
                    )
                },
                |b| {
                    b.ensure_capacity(
                        &gpu.device,
                        instances.len() * std::mem::size_of::<crate::InstanceRaw>(),
                    );
                    b.next();
                    b.write(&gpu.queue, bytemuck::cast_slice(instances));
                },
            );

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, buffer.current().slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..instances.len() as u32);
        }
    }
}

impl Default for ShadowPass {
    fn default() -> Self {
        Self::new()
    }
}
