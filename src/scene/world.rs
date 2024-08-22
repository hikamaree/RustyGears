use cgmath::Vector3;

use crate::{
    SceneRef,
    Scene,
    Shader,
    SceneItem,
};

#[derive(Clone)]
pub struct World {
    pub(crate) scene: SceneRef
}

impl World {
    pub fn new() -> Self {
        Self {
            scene: Scene::create()
        }
    }
    pub fn set_shader(&self, shader: Shader) -> Self{
        self.scene.borrow_mut().shader = Some(shader);
        self.clone()
    }

    pub fn set_depth_shader(&self, shader: Shader) -> Self {
        self.scene.borrow_mut().depth_shader = Some(shader);
        self.clone()
    }

    pub fn add<T: SceneItem>(&self, item: &T) -> Self {
        item.add_to_scene(&mut self.scene.borrow_mut());
        self.clone()
    }

    pub fn set_gravity(&self, gravity: Vector3<f32>) -> Self {
        self.scene.borrow_mut().physics.set_gravity(gravity);
        self.clone()
    }

    pub fn set_physycs_frequency(&self, frequency: f32) -> Self{
        self.scene.borrow_mut().physics.set_refresh_frequency(frequency);
        self.clone()
    }
}
