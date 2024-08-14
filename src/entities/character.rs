use crate::graphics::ModelRef;
use crate::physics::BodyRef;
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
pub struct Character {
    pub models: Vec<ModelRef>,
    pub body: Option<BodyRef>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Character {
    pub fn new() -> Box<dyn Entity> {
        Box::new(Character {
            models: Vec::new(),
            body: None,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        })
    }
}

impl Entity for Character {
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
        if let Some(body) = &self.body {
            self.position = body.borrow().position;
            self.rotation = body.borrow().rotation;
        }
    }

    fn set_body(&mut self, body: BodyRef) -> Box<dyn Entity> {
        body.borrow_mut().movable = true;
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

    fn set_velocity(&mut self, velocity: Vector3<f32>) -> Box<dyn Entity> {
        if let Some(body) = &self.body {
            body.borrow_mut().velocity = velocity;
        }
        Box::new(self.clone())
    }

    fn apply_force(&mut self, force: Vector3<f32>) -> Box<dyn Entity> {
        if let Some(body) = &self.body {
            body.borrow_mut().apply_force(force);
        }
        Box::new(self.clone())
    }

    fn apply_torque(&mut self, torque: Vector3<f32>) -> Box<dyn Entity> {
        if let Some(body) = &self.body {
            body.borrow_mut().apply_torque(torque);
        }
        Box::new(self.clone())
    }
}
