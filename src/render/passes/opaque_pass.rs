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
use crate::render::sort::NoSort;
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
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct OpaquePass {
    sort: NoSort,
    pipeline: RwLock<Option<wgpu::RenderPipeline>>,
    shadow_sampler: RwLock<Option<wgpu::Sampler>>,
    fallback_shadow_bind_group: RwLock<Option<wgpu::BindGroup>>,
}

impl OpaquePass {
    pub fn new() -> Self {
        Self {
            sort: NoSort,
            pipeline: RwLock::new(None),
            shadow_sampler: RwLock::new(None),
            fallback_shadow_bind_group: RwLock::new(None),
        }
    }

    fn ensure_pipeline(
        &self,
        gpu: &GpuResources,
        config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
    ) {
        if self.pipeline.read().unwrap().is_some() {
            return;
        }

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Opaque Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("opaque.wgsl").into()),
            });

        let layouts = resources
            .layouts
            .get_opt::<crate::render::layout::TextureLayout>();
        let texture_layout = match layouts {
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

        let shadow_texture_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::ShadowTextureLayout>()
        {
            Some(l) => l,
            None => return,
        };

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Opaque Pipeline Layout"),
                bind_group_layouts: &[
                    texture_layout,
                    camera_layout,
                    light_layout,
                    shadow_texture_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Opaque Pipeline"),
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
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        *self.pipeline.write().unwrap() = Some(pipeline);

        let shadow_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });

        *self.shadow_sampler.write().unwrap() = Some(shadow_sampler.clone());

        let shadow_texture_layout = match resources
            .layouts
            .get_opt::<crate::render::layout::ShadowTextureLayout>()
        {
            Some(l) => l,
            None => return,
        };

        let dummy_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy Shadow Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let fallback_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fallback Shadow Bind Group"),
            layout: shadow_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        *self.fallback_shadow_bind_group.write().unwrap() = Some(fallback_bind_group);
    }
}

impl RenderPass for OpaquePass {
    fn id(&self) -> PassId {
        PassId::OPAQUE
    }

    fn name(&self) -> &'static str {
        "Opaque"
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
        vec![PassId::SHADOW]
    }

    fn collect(
        &self,
        scene: &WorldScene,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> PassData {
        let view_pos = camera_transform.position;
        let filter = self.filter();

        let mut opaque_data: Vec<(crate::Model3d, usize, usize, crate::InstanceRaw)> = Vec::new();

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

                if matches!(mesh.data.render_tag, crate::RenderTag::Opaque) {
                    opaque_data.push((model3d.clone(), lod_index, mesh_index, transform.raw()));
                }
            }
        }

        let mut group_map: HashMap<(crate::Model3d, usize, usize), Vec<crate::InstanceRaw>> =
            HashMap::new();
        for (model3d, lod, mesh_idx, raw) in opaque_data {
            group_map
                .entry((model3d, lod, mesh_idx))
                .or_default()
                .push(raw);
        }

        let mut model_data: Vec<ModelRenderData> = group_map
            .into_iter()
            .map(|((model3d, lod, mesh_idx), instances)| {
                let instance_count = instances.len() as u32;
                ModelRenderData {
                    model3d,
                    lod_index: lod,
                    instance_data: Arc::from(instances),
                    mesh_ranges: vec![MeshRenderRange {
                        mesh_index: mesh_idx,
                        visible_instance_ranges: vec![0..instance_count],
                    }],
                }
            })
            .collect();

        self.sort.sort(&mut model_data, view_pos);

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
        self.ensure_pipeline(gpu, config, resources);

        let pipeline_guard = self.pipeline.read().unwrap();
        let pipeline = match pipeline_guard.as_ref() {
            Some(p) => p,
            None => return,
        };

        let shadow_bind_group = match (
            ctx.get_input(PassId::SHADOW),
            self.shadow_sampler.read().unwrap().as_ref(),
        ) {
            (Some(shadow_view), Some(shadow_sampler)) => {
                let layout = resources
                    .layouts
                    .get_opt::<crate::render::layout::ShadowTextureLayout>()
                    .unwrap();
                Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Shadow Bind Group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(shadow_sampler),
                        },
                    ],
                }))
            }
            _ => None,
        };

        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Opaque Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.screen_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(ctx.clear_color()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: ctx.depth_view,
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
        render_pass.set_bind_group(1, Some(ctx.camera_bind_group), &[]);
        render_pass.set_bind_group(2, Some(ctx.light_bind_group), &[]);

        if let Some(shadow_bg) = shadow_bind_group.as_ref() {
            render_pass.set_bind_group(3, Some(shadow_bg), &[]);
        } else if let Some(fallback) = self.fallback_shadow_bind_group.read().unwrap().as_ref() {
            render_pass.set_bind_group(3, Some(fallback), &[]);
        }

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

                render_pass.set_vertex_buffer(1, buffer.current().slice(..));
                *ctx.frame_counter += render_pass.draw_mesh_instanced(
                    &model.meshes[mesh_range.mesh_index],
                    &model.materials[model.meshes[mesh_range.mesh_index].material],
                    ctx.camera_bind_group,
                    ctx.light_bind_group,
                    mesh_range,
                );
            }
        }
    }
}

impl Default for OpaquePass {
    fn default() -> Self {
        Self::new()
    }
}
