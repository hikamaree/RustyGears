use super::body::*;
use cgmath::Vector3;

pub struct PhysicsWorld {
    pub bodies: Vec<RigidBody>,
    pub gravity: Vector3<f32>,
}

impl PhysicsWorld {
    pub fn new(gravity: Vector3<f32>) -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity,
        }
    }

    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }

    pub fn update(&mut self, dt: f32) {
        for body in &mut self.bodies {
            body.apply_force(self.gravity * body.mass);
            body.update(dt);
        }
    }
}
