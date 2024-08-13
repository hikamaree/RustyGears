use crate::graphics::ModelRef;
use crate::physics::BodyRef;

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
    pub fn new() -> Self {
        Character {
            models: Vec::new(),
            body: None,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        }
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        if let Some(body) = &self.body {
            body.borrow_mut().position = position;
        }
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
        if let Some(body) = &self.body {
            body.borrow_mut().rotation = rotation;
        }
    }

    pub fn set_velocity(&mut self, velocity: Vector3<f32>) {
        if let Some(body) = &self.body {
            body.borrow_mut().velocity = velocity;
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) {
        if let Some(body) = &self.body {
            body.borrow_mut().apply_force(force);
        }
    }

    pub fn apply_torque(&mut self, force: Vector3<f32>) {
        if let Some(body) = &self.body {
            body.borrow_mut().apply_torque(force);
        }
    }
}
