use cgmath::{Vector3, Zero, Point3, EuclideanSpace};
use super::collision_box::*;
use crate::model::*;

use core::cell::RefCell;
use std::rc::Rc;

pub struct RigidBody {
    pub position: Vector3<f32>,
    pub previous_position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub mass: f32,
    pub forces: Vector3<f32>,
    pub gravity: bool,
    pub movable: bool,
    pub collision_box: Vec<CollisionBox>,
}

pub type BodyRef = Rc<RefCell<RigidBody>>;

impl RigidBody {
    pub fn new(position: Vector3<f32>, mass: f32, collision_box: Vec<CollisionBox>) -> BodyRef {
        let mut body: RigidBody = RigidBody {
            position,
            previous_position: position,
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            mass,
            forces: Vector3::zero(),
            gravity: true,
            movable: true,
            collision_box,
        };
        body.update_collision_shapes();
        Rc::new(RefCell::new(body))
    }

    pub fn from_model_with_bounding_boxes(model: &Model, mass: f32) -> BodyRef {
        let collision_box = model.meshes.iter().map(|mesh| {
            CollisionBox::BoundingBox(mesh.calculate_bounding_box())
        }).collect();

        RigidBody::new(model.position, mass, collision_box)
    }

    pub fn from_model_with_spheres(model: &Model, mass: f32) -> BodyRef {
        let collision_box = model.meshes.iter().map(|mesh| {
            CollisionBox::Sphere(mesh.calculate_sphere())
        }).collect();

        RigidBody::new(model.position, mass, collision_box)
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) {
        self.forces += force;
    }

    pub fn ignore_gravity(&mut self) {
        self.gravity = false;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.movable { return; }
        self.previous_position = self.position;
        self.acceleration = self.forces / self.mass;
        self.position += self.velocity * dt + self.acceleration * dt * dt / 2.0;
        self.velocity += self.acceleration * dt;
        self.forces = Vector3::zero();
        self.update_collision_shapes();
    }

    fn update_collision_shapes(&mut self) {
        for shape in &mut self.collision_box {
            match shape {
                CollisionBox::BoundingBox(bbox) => {
                    let size = bbox.max - bbox.min;
                    bbox.min = Point3::from_vec(self.position - size * 0.5);
                    bbox.max = Point3::from_vec(self.position + size * 0.5);
                }
                CollisionBox::Sphere(sphere) => {
                    sphere.center = Point3::from_vec(self.position);
                }
            }
        }
    }
}
