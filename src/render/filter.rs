use crate::Entity;
use crate::WorldScene;

#[derive(Clone)]
pub struct EntityFilter {
    requires: Vec<std::any::TypeId>,
    excludes: Vec<std::any::TypeId>,
}

impl EntityFilter {
    pub fn new() -> Self {
        Self {
            requires: Vec::new(),
            excludes: Vec::new(),
        }
    }

    pub fn with_component<C: 'static>(mut self) -> Self {
        self.requires.push(std::any::TypeId::of::<C>());
        self
    }

    pub fn exclude<C: 'static>(mut self) -> Self {
        self.excludes.push(std::any::TypeId::of::<C>());
        self
    }

    pub fn matches(&self, scene: &WorldScene, entity: Entity) -> bool {
        for &tid in &self.requires {
            let has = scene.world.has_by_id(entity, tid);
            if !has {
                return false;
            }
        }

        for &tid in &self.excludes {
            if scene.world.has_by_id(entity, tid) {
                return false;
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.requires.is_empty() && self.excludes.is_empty()
    }
}

impl Default for EntityFilter {
    fn default() -> Self {
        Self::new()
    }
}
