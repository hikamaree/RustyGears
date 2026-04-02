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
use crate::BufferKey;
use crate::Camera;
use crate::DrawModel;
use crate::MeshRenderRange;
use crate::ModelRenderData;
use crate::RenderObject;
use crate::Transform;
use crate::Vertex;
use crate::WorldScene;
use std::sync::Arc;
use std::sync::RwLock;

pub struct TransparentPass {
    sort: DepthSortBackToFront,
    pipeline: RwLock<Option<wgpu::RenderPipeline>>,
}

impl TransparentPass {
    pub fn new() -> Self {
        Self {
            sort: DepthSortBackToFront,
            pipeline: RwLock::new(None),
        }
    }

    fn ensure_pipeline(
        &self,
        gpu: &GpuResources,
        config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
    ) {
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
                label: Some("Transparent Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("transparent.wgsl").into()),
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
                label: Some("Transparent Pipeline Layout"),
                bind_group_layouts: &[texture_layout, camera_layout, light_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Transparent Pipeline"),
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
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Less,
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

impl RenderPass for TransparentPass {
    fn id(&self) -> PassId {
        PassId::TRANSPARENT
    }

    fn name(&self) -> &'static str {
        "Transparent"
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
        vec![]
    }

    fn inputs(&self) -> Vec<PassId> {
        vec![PassId::OPAQUE, PassId::WEIGHTED]
    }

    fn collect(
        &self,
        scene: &WorldScene,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> PassData {
        let view_pos = camera_transform.position;
        let view_matrix = camera.calc_matrix(camera_transform);
        let filter = self.filter();

        let mut transparent_data: Vec<(f32, crate::Model3d, usize, usize, crate::InstanceRaw)> =
            Vec::new();

        for (entity, render_obj, transform) in scene
            .world
            .query2::<RenderObject, crate::Transform>()
            .iter()
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
            let scale = calculate_model_scale(&model_mat);

            for (mesh_index, mesh) in lod_model.meshes.iter().enumerate() {
                let center_local: cgmath::Vector4<f32> =
                    mesh.data.bounding_sphere.center.extend(1.0);
                let center_world: cgmath::Vector4<f32> = model_mat * center_local;
                let radius = mesh.data.bounding_sphere.radius * scale;

                if !camera.can_see4(center_world, radius) {
                    continue;
                }

                if matches!(mesh.data.render_tag, crate::RenderTag::SortedTransparent) {
                    let center_view: cgmath::Vector4<f32> = view_matrix * center_world;
                    let depth = -center_view.z;
                    transparent_data.push((
                        depth,
                        model3d.clone(),
                        lod_index,
                        mesh_index,
                        transform.raw(),
                    ));
                }
            }
        }

        transparent_data.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let model_data: Vec<ModelRenderData> = transparent_data
            .into_iter()
            .map(|(_, model3d, lod, mesh_idx, raw)| ModelRenderData {
                model3d,
                lod_index: lod,
                instance_data: Arc::from([raw]),
                mesh_ranges: vec![MeshRenderRange {
                    mesh_index: mesh_idx,
                    visible_instance_ranges: vec![0..1],
                }],
            })
            .collect();

        PassData { model_data }
    }

    fn execute(
        &self,
        ctx: &mut PassContext,
        gpu: &GpuResources,
        config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
        data: &PassData,
    ) {
        if data.model_data.is_empty() {
            return;
        }

        self.ensure_pipeline(gpu, config, resources);

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

        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Transparent Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.screen_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
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

        render_pass.set_pipeline(pipeline);
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
                    &key_str,
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
                *ctx.frame_counter += render_pass.draw_mesh_instanced(
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

impl Default for TransparentPass {
    fn default() -> Self {
        Self::new()
    }
}
