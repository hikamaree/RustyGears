use crate::render::filter::EntityFilter;
use crate::render::gpu_resources::GpuResources;
use crate::render::pass_id::PassId;
use crate::render::render_resources::RenderResources;
use crate::render::render_state::RenderState;
use crate::render::sort::SortStrategy;
use crate::render::targets::RenderTargetPool;
use crate::render::RenderConfig;
use crate::Camera;
use crate::GpuLight;
use crate::ModelRenderData;
use crate::Transform;
use crate::WorldScene;
use std::collections::HashMap;

pub struct PassContext<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub scene: &'a WorldScene,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub shadow_camera_bind_group: &'a wgpu::BindGroup,
    pub lights: &'a [GpuLight],
    pub targets: &'a mut RenderTargetPool,
    pub frame_counter: &'a mut u32,
    pub screen_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub screen_size: (u32, u32),
    pub input_views: &'a mut HashMap<PassId, wgpu::TextureView>,
    pub config: &'a RenderConfig,
}

impl<'a> PassContext<'a> {
    pub fn get_input(&self, id: PassId) -> Option<&wgpu::TextureView> {
        self.input_views.get(&id)
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.config.clear_color
    }

    pub fn shadow_config(&self) -> (u32, f32, f32, f32) {
        (
            self.config.shadow_cascade_resolution,
            self.config.shadow_frustum_size,
            self.config.shadow_frustum_near,
            self.config.shadow_frustum_far,
        )
    }

    pub fn shadow_cascade_resolution(&self) -> u32 {
        self.config.shadow_cascade_resolution
    }
}

#[derive(Debug, Default)]
pub struct PassData {
    pub model_data: Vec<ModelRenderData>,
}

impl Clone for PassData {
    fn clone(&self) -> Self {
        Self {
            model_data: self.model_data.clone(),
        }
    }
}

pub trait RenderPass: Send + Sync + 'static {
    fn id(&self) -> PassId;

    fn name(&self) -> &'static str {
        self.id().as_str()
    }

    fn filter(&self) -> &EntityFilter;

    fn sort(&self) -> &dyn SortStrategy;

    fn outputs(&self) -> Vec<PassOutput> {
        vec![]
    }

    fn inputs(&self) -> Vec<PassId> {
        vec![]
    }

    fn should_run(&self, _scene: &WorldScene) -> bool {
        true
    }

    fn setup(
        &self,
        _gpu: &GpuResources,
        _config: &wgpu::SurfaceConfiguration,
        _resources: &mut RenderResources,
        _state: &mut RenderState,
    ) {
    }

    fn collect(
        &self,
        scene: &WorldScene,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> PassData;

    fn execute(
        &self,
        ctx: &mut PassContext,
        gpu: &GpuResources,
        config: &wgpu::SurfaceConfiguration,
        resources: &RenderResources,
        data: &PassData,
    );
}

#[derive(Debug, Clone)]
pub struct PassOutput {
    pub id: PassId,
    pub descriptor: crate::render::targets::TargetDescriptor,
}

impl PassData {
    pub fn empty() -> Self {
        Self::default()
    }
}
