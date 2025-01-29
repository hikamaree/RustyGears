use crate::entities::*;
use crate::graphics::*;
use crate::scene::*;

pub trait SceneItem {
    fn add_to_scene(&self, scene: &mut Scene);
}

impl<T: Entity + Clone + 'static> SceneItem for T {
    fn add_to_scene(&self, scene: &mut Scene) {
        scene.add_entity(self.clone());
    }
}

impl SceneItem for AmbientLight {
    fn add_to_scene(&self, scene: &mut Scene) {
        scene.set_ambient_light(self.clone());
    }
}

impl SceneItem for LightSource {
    fn add_to_scene(&self, scene: &mut Scene) {
        scene.add_light_source(self.clone());
    }
}

impl SceneItem for Fog {
    fn add_to_scene(&self, scene: &mut Scene) {
        scene.set_fog(self.clone());
    }
}
