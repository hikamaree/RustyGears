use crate::render::filter::EntityFilter;
use crate::render::gpu_resources::GpuResources;
use crate::render::pass_id::PassId;
use crate::render::render_resources::RenderResources;
use crate::render::render_state::RenderState;
use crate::render::sort::SortStrategy;
use crate::render::targets::RenderTargetPool;
use crate::Camera;
use crate::GpuLight;
use crate::ModelRenderData;
use crate::Transform;
use crate::WorldScene;

pub struct PassContext<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub scene: &'a WorldScene,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub shadow_camera_bind_group: &'a wgpu::BindGroup,
    pub lights: &'a [GpuLight],
    pub targets: &'a mut RenderTargetPool,
    pub screen_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub screen_size: (u32, u32),
    pub shadow_view: Option<wgpu::TextureView>,
}

#[derive(Debug, Default)]
pub struct PassData {
    pub model_data: Vec<ModelRenderData>,
}

pub trait RenderPass: Send + Sync + 'static {
    fn id(&self) -> PassId;

    fn name(&self) -> &'static str {
        self.id().as_str()
    }

    fn order(&self) -> u32 {
        100
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
        state: &mut RenderState,
        data: &PassData,
    );
}

#[derive(Clone)]
pub struct PassOutput {
    pub id: PassId,
    pub descriptor: crate::render::targets::TargetDescriptor,
}

impl PassData {
    pub fn new() -> Self {
        Self::default()
    }
}
