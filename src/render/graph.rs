use crate::render::pass::PassOutput;
use crate::render::pass::RenderPass;
use crate::render::pass_id::PassId;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct GraphNode {
    pub pass: Arc<dyn RenderPass>,
    pub outputs: Vec<PassOutput>,
    pub inputs: Vec<PassId>,
}

#[derive(Debug)]
pub enum GraphError {
    Cycle(Vec<PassId>),
    MissingInput(PassId, PassId),
    MultipleScreenPasses(Vec<PassId>),
    ZeroScreenPasses,
    DuplicatePass(PassId),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::Cycle(passes) => {
                write!(f, "Render graph cycle detected: {:?}", passes)
            }
            GraphError::MissingInput(pass, missing) => {
                write!(f, "Pass '{}' requires missing input '{}'", pass, missing)
            }
            GraphError::MultipleScreenPasses(passes) => {
                write!(f, "Multiple passes write to screen: {:?}", passes)
            }
            GraphError::ZeroScreenPasses => {
                write!(f, "No pass writes to screen")
            }
            GraphError::DuplicatePass(id) => {
                write!(f, "Pass '{}' already registered", id)
            }
        }
    }
}

impl std::error::Error for GraphError {}

pub struct RenderGraph {
    nodes: BTreeMap<PassId, GraphNode>,
    execution_order: Vec<PassId>,
    screen_pass: Option<PassId>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            execution_order: Vec::new(),
            screen_pass: None,
        }
    }

    pub fn add_pass(&mut self, pass: Arc<dyn RenderPass>) -> Result<(), GraphError> {
        let id = pass.id();

        if self.nodes.contains_key(&id) {
            return Err(GraphError::DuplicatePass(id));
        }

        let pass_clone = pass.clone();
        let node = GraphNode {
            pass: pass_clone,
            outputs: pass.outputs(),
            inputs: pass.inputs(),
        };

        self.nodes.insert(id, node);
        self.rebuild_execution_order()?;

        Ok(())
    }

    pub fn validate(&mut self) -> Result<(), GraphError> {
        self.validate_screen_passes()
    }

    fn rebuild_execution_order(&mut self) -> Result<(), GraphError> {
        let mut in_degree: BTreeMap<PassId, usize> = BTreeMap::new();
        let mut adjacency: BTreeMap<PassId, Vec<PassId>> = BTreeMap::new();

        for (id, node) in &self.nodes {
            in_degree.insert(*id, node.inputs.len());
            adjacency.entry(*id).or_default();

            for input_id in &node.inputs {
                if !self.nodes.contains_key(input_id) {
                    return Err(GraphError::MissingInput(*id, *input_id));
                }
                adjacency.entry(*input_id).or_default().push(*id);
            }
        }

        let mut queue: VecDeque<PassId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut order = Vec::new();

        while let Some(pass_id) = queue.pop_front() {
            order.push(pass_id);

            if let Some(dependents) = adjacency.get(&pass_id) {
                for dep in dependents {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(*dep);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            let remaining: Vec<PassId> = self
                .nodes
                .keys()
                .filter(|id| !order.contains(id))
                .copied()
                .collect();
            return Err(GraphError::Cycle(remaining));
        }

        self.execution_order = order;
        Ok(())
    }

    fn validate_screen_passes(&mut self) -> Result<(), GraphError> {
        let screen_passes: Vec<PassId> = self
            .nodes
            .iter()
            .filter(|(_, node)| node.outputs.is_empty())
            .map(|(id, _)| *id)
            .collect();

        match screen_passes.len() {
            0 => Err(GraphError::ZeroScreenPasses),
            1 => {
                self.screen_pass = Some(screen_passes[0]);
                Ok(())
            }
            _ => Err(GraphError::MultipleScreenPasses(screen_passes)),
        }
    }

    pub fn get_node(&self, id: &PassId) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    pub fn get_outputs(&self, id: &PassId) -> Option<&Vec<PassOutput>> {
        self.nodes.get(id).map(|n| &n.outputs)
    }

    pub fn execution_order(&self) -> &[PassId] {
        &self.execution_order
    }

    pub fn screen_pass(&self) -> Option<PassId> {
        self.screen_pass
    }

    pub fn pass_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn inputs_of(&self, pass_id: &PassId) -> &[PassId] {
        self.nodes
            .get(pass_id)
            .map(|n| n.inputs.as_slice())
            .unwrap_or(&[])
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}
