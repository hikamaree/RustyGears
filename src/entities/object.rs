use crate::Model;
use crate::BodyRef;
use crate::Entity;

use crate::PhysicsWorld;
use crate::Shader;

use cgmath::{
    Vector3,
    Quaternion,
    Zero,
    One
};

#[derive(Clone)]
pub struct Object {
    pub(super) models: Vec<Model>,
    pub(super) body: Option<BodyRef>,
    pub(super) position: Vector3<f32>,
    pub(super) rotation: Quaternion<f32>,
}

impl Object {
    pub fn new() -> Self {
        Object {
            models: Vec::new(),
            body: None,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        }
    }

    pub fn set_mass(&mut self, mass: f32) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().set_mass(mass);
        }
        self.clone()
    }

    pub fn set_body(&mut self, body: BodyRef) -> Self {
        body.lock().unwrap().movable = false;
        self.body = Some(body);
        self.clone()
    }

    pub fn add_model(&mut self, model: Model) -> Self {
        self.models.push(model);
        self.clone()
    }

    pub fn set_position(&mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        if let Some(body) = &self.body {
            body.lock().unwrap().set_position(position);
        }
        self.clone()
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) -> Self {
        self.rotation = rotation;
        if let Some(body) = &self.body {
            body.lock().unwrap().rotation = rotation;
        }
        self.clone()
    }
}

impl Entity for Object {
        fn clone_entity(&self) -> Box<dyn Entity> {
        Box::new(self.clone())
    }

    fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.draw(shader, self.position, self.rotation);
        }
    }

    fn set_physics(&self, world: &mut PhysicsWorld) {
        if let Some(body) = &self.body {
            world.add_body(body.clone());
        }
    }

    fn update(&mut self) {
    }
}
