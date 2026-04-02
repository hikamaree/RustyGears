use crate::math::Matrix4;
use crate::render::cull::calculate_model_scale;
use crate::render::filter::EntityFilter;
use crate::render::gpu_resources::GpuResources;
use crate::render::lod::select_lod_by_position;
use crate::render::pass::PassContext;
use crate::render::pass::PassData;
use crate::render::pass::PassOutput;
use crate::render::pass::RenderPass;
use crate::render::pass_id::PassId;
use crate::render::render_resources::RenderResources;
use crate::render::sort::DepthSortBackToFront;
use crate::render::sort::SortStrategy;
use crate::render::targets::TargetDescriptor;
use crate::render::targets::TargetSize;
use crate::BufferKey;
use crate::Camera;
use crate::DrawModel;
use crate::ModelRenderData;
use crate::RenderObject;
use crate::Transform;
use crate::Vertex;
use crate::WorldScene;
use std::sync::Arc;
use std::sync::RwLock;

pub struct WeightedPass {
    sort: DepthSortBackToFront,
    pipeline: RwLock<Option<wgpu::RenderPipeline>>,
}

impl WeightedPass {
    pub fn new() -> Self {
        Self {
            sort: DepthSortBackToFront,
            pipeline: RwLock::new(None),
        }
    }

    fn ensure_pipeline(&self, gpu: &GpuResources, resources: &RenderResources) {
        if self
            .pipeline
            .read()
            .ok()
            .map(|p| p.is_some())
            .unwrap_or(false)
        {
            return;
        }

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Weighted Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("weighted.wgsl").into()),
            });

        let texture_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::TextureLayout>()
        {
            Some(l) => l,
            None => return,
        };
        let camera_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::CameraLayout>()
        {
            Some(l) => l,
            None => return,
        };
        let light_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::LightLayout>()
        {
            Some(l) => l,
            None => return,
        };

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Weighted Pipeline Layout"),
                bind_group_layouts: &[texture_layout, camera_layout, light_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Weighted Pipeline"),
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
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: crate::Texture::ACCUM_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: crate::Texture::REVEALAGE_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::Zero,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::Zero,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        if let Ok(mut pipeline_guard) = self.pipeline.write() {
            *pipeline_guard = Some(pipeline);
        } else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire pipeline write lock"
            );
        }
    }
}

impl RenderPass for WeightedPass {
    fn id(&self) -> PassId {
        PassId::WEIGHTED
    }

    fn name(&self) -> &'static str {
        "Weighted"
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
        vec![
            PassOutput {
                id: PassId::WEIGHTED_ACCUM,
                descriptor: TargetDescriptor {
                    format: crate::Texture::ACCUM_FORMAT,
                    size: TargetSize::Screen,
                    sample_count: 1,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                },
            },
            PassOutput {
                id: PassId::WEIGHTED_REVEALAGE,
                descriptor: TargetDescriptor {
                    format: crate::Texture::REVEALAGE_FORMAT,
                    size: TargetSize::Screen,
                    sample_count: 1,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                },
            },
        ]
    }

    fn inputs(&self) -> Vec<PassId> {
        vec![PassId::OPAQUE]
    }

    fn collect(
        &self,
        scene: &WorldScene,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> PassData {
        let view_pos = camera_transform.position;
        let filter = self.filter();

        let mut weighted_data: Vec<ModelRenderData> = Vec::new();

        for (entity, render_obj, transform) in
            scene.world.query2::<RenderObject, Transform>().iter()
        {
            if !filter.matches(scene, *entity) {
                continue;
            }

            if render_obj.lods.is_empty() {
                continue;
            }

            let lod_index =
                select_lod_by_position(view_pos, transform.position, render_obj.lods.len());

            let model3d = render_obj.lods[lod_index].clone();

            let Some(lod_model) = scene.get_model3d(&model3d) else {
                continue;
            };

            let model_mat = Matrix4::from(transform.raw().model);

            for (mesh_index, mesh) in lod_model.meshes.iter().enumerate() {
                if !matches!(mesh.data.render_tag, crate::RenderTag::WeightedTransparent) {
                    continue;
                }

                let center_local: cgmath::Vector4<f32> =
                    mesh.data.bounding_sphere.center.extend(1.0);
                let center_world: cgmath::Vector4<f32> = model_mat * center_local;
                let scale = calculate_model_scale(&model_mat);
                let radius = mesh.data.bounding_sphere.radius * scale;

                if !camera.can_see(
                    cgmath::Vector3::new(center_world.x, center_world.y, center_world.z),
                    radius,
                ) {
                    continue;
                }

                weighted_data.push(ModelRenderData {
                    instance_data: Arc::new([transform.raw()]),
                    mesh_ranges: vec![crate::MeshRenderRange {
                        mesh_index,
                        visible_instance_ranges: vec![0..1],
                    }],
                    model3d: model3d.clone(),
                    lod_index,
                });
            }
        }

        PassData {
            model_data: weighted_data,
        }
    }

    fn execute(
        &self,
        ctx: &mut PassContext,
        gpu: &GpuResources,
        _config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
        data: &PassData,
    ) {
        self.ensure_pipeline(gpu, resources);

        let Ok(pipeline_guard) = self.pipeline.read() else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire pipeline read lock"
            );
            return;
        };
        let pipeline = match pipeline_guard.as_ref() {
            Some(p) => p,
            None => return,
        };

        let screen_width = ctx.screen_size.0;
        let screen_height = ctx.screen_size.1;

        let accum_target = ctx.targets.allocate(
            &gpu.device,
            &TargetDescriptor {
                format: crate::Texture::ACCUM_FORMAT,
                size: TargetSize::Screen,
                sample_count: 1,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
            screen_width,
            screen_height,
        );

        let revealage_target = ctx.targets.allocate(
            &gpu.device,
            &TargetDescriptor {
                format: crate::Texture::REVEALAGE_FORMAT,
                size: TargetSize::Screen,
                sample_count: 1,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
            screen_width,
            screen_height,
        );

        let accum_view = match ctx.targets.get_view(accum_target) {
            Some(v) => v,
            None => return,
        };

        let revealage_view = match ctx.targets.get_view(revealage_target) {
            Some(v) => v,
            None => return,
        };

        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Weighted Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: accum_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: revealage_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: ctx.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(1, Some(ctx.camera_bind_group), &[]);
        render_pass.set_bind_group(2, Some(ctx.light_bind_group), &[]);

        for model_data in &data.model_data {
            let Some(model) = ctx.scene.models3d.get(&model_data.model3d) else {
                continue;
            };

            for mesh_range in &model_data.mesh_ranges {
                let key = BufferKey::new(
                    model_data.model3d.clone(),
                    model_data.lod_index,
                    mesh_range.mesh_index,
                );
                let key_str = key.to_string();

                let instance_count = mesh_range
                    .visible_instance_ranges
                    .iter()
                    .map(|r| r.end - r.start)
                    .sum::<u32>() as usize;

                let buffer = resources.get_or_create_buffer_with(
                    &gpu.queue,
                    key_str.as_str(),
                    || {
                        crate::Buffer::new(
                            &gpu.device,
                            instance_count.next_power_of_two()
                                * std::mem::size_of::<crate::InstanceRaw>(),
                            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            crate::BufferStrategy::Triple,
                            &key_str,
                        )
                    },
                    |b| {
                        b.ensure_capacity(
                            &gpu.device,
                            instance_count * std::mem::size_of::<crate::InstanceRaw>(),
                        );
                        b.next();
                        b.write(&gpu.queue, bytemuck::cast_slice(&model_data.instance_data));
                    },
                );

                let mesh = &model.meshes[mesh_range.mesh_index];
                let material = &model.materials[mesh.material];

                render_pass.set_vertex_buffer(1, buffer.current().slice(..));
                render_pass.draw_mesh_instanced(
                    mesh,
                    material,
                    ctx.camera_bind_group,
                    ctx.light_bind_group,
                    mesh_range,
                );
            }
        }
    }
}

impl Default for WeightedPass {
    fn default() -> Self {
        Self::new()
    }
}
