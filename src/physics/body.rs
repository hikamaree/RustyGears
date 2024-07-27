use cgmath::{Vector3, Zero};

pub struct RigidBody {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub mass: f32,
    pub forces: Vector3<f32>,
}

impl RigidBody {
    pub fn new(position: Vector3<f32>, mass: f32) -> Self {
        RigidBody {
            position,
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            mass,
            forces: Vector3::zero(),
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) {
        self.forces += force;
    }

    pub fn update(&mut self, dt: f32) {
        self.acceleration = self.forces / self.mass;
        self.velocity += self.acceleration * dt;
        self.position += self.velocity * dt;
        self.forces = Vector3::zero();
    }
}
