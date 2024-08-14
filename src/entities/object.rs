use crate::ModelRef;
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
    pub(super) models: Vec<ModelRef>,
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
}

impl Entity for Object {
        fn clone_entity(&self) -> Box<dyn Entity> {
        Box::new(self.clone())
    }

    fn draw(&self, shader: &Shader) {
        for model in &self.models {
            model.borrow().draw(shader, self.position, self.rotation);
        }
    }

    fn set_physics(&mut self, world: &mut PhysicsWorld) -> Box<dyn Entity> {
        if let Some(body) = &self.body {
            world.add_body(body.clone());
        }
        Box::new(self.clone())
    }

    fn update(&mut self) {
    }

    fn set_body(&mut self, body: BodyRef) -> Box<dyn Entity> {
        body.borrow_mut().movable = false;
        self.body = Some(body);
        Box::new(self.clone())
    }

    fn add_model(&mut self, model: ModelRef) -> Box<dyn Entity> {
        self.models.push(model);
        Box::new(self.clone())
    }


    fn set_position(&mut self, position: Vector3<f32>) -> Box<dyn Entity> {
        self.position = position;
        if let Some(body) = &self.body {
            body.borrow_mut().position = position;
        }
        Box::new(self.clone())
    }

    fn set_rotation(&mut self, rotation: Quaternion<f32>) -> Box<dyn Entity> {
        self.rotation = rotation;
        if let Some(body) = &self.body {
            body.borrow_mut().rotation = rotation;
        }
        Box::new(self.clone())
    }

    fn set_velocity(&mut self, _velocity: Vector3<f32>) -> Box<dyn Entity> {
        Box::new(self.clone())
    }

    fn apply_force(&mut self, _force: Vector3<f32>) -> Box<dyn Entity> {
        Box::new(self.clone())
    }

    fn apply_torque(&mut self, _torque: Vector3<f32>) -> Box<dyn Entity> {
        Box::new(self.clone())
    }
}
