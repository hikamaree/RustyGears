use crate::graphics::Model;
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
    pub models: Vec<Model>,
    pub body: Option<BodyRef>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Character {
    pub fn new() -> Self {
        Character {
            models: Vec::new(),
            body: None,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        }
    }

    pub fn set_body(&mut self, body: BodyRef) -> Self {
        body.lock().unwrap().movable = true;
        self.body = Some(body);
        self.clone()
    }

    pub fn set_mass(&mut self, mass: f32) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().set_mass(mass);
        }
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

    pub fn set_velocity(&mut self, velocity: Vector3<f32>) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().velocity = velocity;
        }
        self.clone()
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().apply_force(force);
        }
        self.clone()
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().apply_torque(torque);
        }
        self.clone()
    }

    pub fn set_gravity(&mut self, gravity: bool) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().set_gravity(gravity);
        }
        self.clone()
    }

    pub fn set_bounciness(&mut self, bounciness: f32) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().set_bounciness(bounciness);
        }
        self.clone()
    }

    pub fn set_friction_coefficient(&mut self, friction_coefficient: f32) -> Self {
        if let Some(body) = &self.body {
            body.lock().unwrap().set_friction_coefficient(friction_coefficient);
        }
        self.clone()
    }

}

impl Entity for Character {
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
        if let Some(body) = &self.body {
            self.position = body.lock().unwrap().position;
            self.rotation = body.lock().unwrap().rotation;
        }
    }
}
