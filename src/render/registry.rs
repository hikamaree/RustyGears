use crate::render::graph::RenderGraph;
use crate::render::pass::RenderPass;
use crate::render::pass_id::PassId;
use crate::Command;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct PassRegistry {
    passes: BTreeMap<PassId, Arc<dyn RenderPass>>,
    bind_groups: BTreeMap<BindGroupId, wgpu::BindGroup>,
    graph: RenderGraph,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum BindGroupId {
    Camera,
    Light,
    ShadowCamera,
}

impl PassRegistry {
    pub fn new() -> Self {
        Self {
            passes: BTreeMap::new(),
            bind_groups: BTreeMap::new(),
            graph: RenderGraph::new(),
        }
    }

    pub fn register_pass(
        &mut self,
        pass: Arc<dyn RenderPass>,
    ) -> Result<(), crate::render::graph::GraphError> {
        let id = pass.id();
        self.passes.insert(id, pass.clone());
        self.graph.add_pass(pass)?;
        Ok(())
    }

    pub fn validate(&mut self) -> Result<(), crate::render::graph::GraphError> {
        self.graph.validate()
    }

    pub fn get_pass(&self, id: &PassId) -> Option<Arc<dyn RenderPass>> {
        self.passes.get(id).cloned()
    }

    pub fn iter_passes(&self) -> Vec<Arc<dyn RenderPass>> {
        self.passes.values().cloned().collect()
    }

    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn register_bind_group(&mut self, id: BindGroupId, bind_group: wgpu::BindGroup) {
        self.bind_groups.insert(id, bind_group);
    }

    pub fn get_bind_group(&self, id: BindGroupId) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(&id)
    }

    pub fn graph(&self) -> &RenderGraph {
        &self.graph
    }
}

impl Default for PassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegisterPass {
    pub render_pass: Arc<dyn RenderPass>,
}

impl Command for RegisterPass {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        let Ok(state) = game.components.get_mut::<crate::render::RenderState>() else {
            crate::log!(crate::LogKind::Error, "RenderState is not initialized!");
            return;
        };
        let mut registry = state.registry.write().unwrap();
        if let Err(e) = registry.register_pass(self.render_pass) {
            crate::log!(crate::LogKind::Error, "Failed to register pass: {}", e);
        }
    }
}
