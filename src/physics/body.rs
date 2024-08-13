use cgmath::{
    Vector3,
    Point3,
    Matrix3,
    Quaternion,
    Rotation3,
    Zero,
    Rad,
    EuclideanSpace,
    SquareMatrix,
    InnerSpace,
};
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

    pub rotation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub angular_acceleration: Vector3<f32>,
    pub torque: Vector3<f32>,
    pub inertia_tensor: Matrix3<f32>,
    pub inverse_inertia_tensor: Matrix3<f32>,

    pub bounciness: f32,
    pub friction_coefficient: f32,
}

pub type BodyRef = Rc<RefCell<RigidBody>>;

impl RigidBody {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>,mass: f32, collision_box: Vec<CollisionBox>) -> BodyRef {
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
            rotation,
            angular_velocity: Vector3::zero(),
            angular_acceleration: Vector3::zero(),
            torque: Vector3::zero(),
            inertia_tensor: Matrix3::identity(),
            inverse_inertia_tensor: Matrix3::identity(),
            bounciness: 0.5,
            friction_coefficient: 1.0,
        };

        body.update_collision_shapes();
        body.update_inertia_tensor();
        Rc::new(RefCell::new(body))
    }

    pub fn from_model_with_bounding_boxes(model: &Model, mass: f32) -> BodyRef {
        let collision_box = model.meshes.iter().map(|mesh| {
            CollisionBox::BoundingBox(mesh.calculate_bounding_box())
        }).collect();

        RigidBody::new(model.position, model.rotation, mass, collision_box)
    }

    pub fn from_model_with_spheres(model: &Model, mass: f32) -> BodyRef {
        let collision_box = model.meshes.iter().map(|mesh| {
            CollisionBox::Sphere(mesh.calculate_sphere())
        }).collect();

        RigidBody::new(model.position, model.rotation, mass, collision_box)
    }

    pub fn set_bounciness(&mut self, bounciness: f32) {
        if bounciness > 1.0 {
            self.bounciness = 1.0;
        } else if bounciness < 0.0 {
            self.bounciness = 0.0;
        } else {
            self.bounciness = bounciness;
        }
    }

    pub fn set_friction_coefficient(&mut self, friction_coefficient: f32) {
        if friction_coefficient < 0.0 {
            self.friction_coefficient = 0.0;
        } else {
            self.friction_coefficient = friction_coefficient;
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) {
        self.forces += force;
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>) {
        self.torque += torque;
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

        self.angular_acceleration = self.inverse_inertia_tensor * self.torque;
        self.angular_velocity += self.angular_acceleration * dt;

        if self.angular_velocity.magnitude() < 0.1 {
            self.angular_velocity = Vector3::zero();
        } else {
            let delta_rotation = Quaternion::from_angle_x(Rad(self.angular_velocity.x * dt)) *
                Quaternion::from_angle_y(Rad(self.angular_velocity.y * dt)) *
                Quaternion::from_angle_z(Rad(self.angular_velocity.z * dt));
            self.rotation = (delta_rotation * self.rotation).normalize();
        }

        self.torque = Vector3::zero();

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

    fn update_inertia_tensor(&mut self) {
        let mut inertia_tensor = Matrix3::zero();

        for shape in &self.collision_box {
            match shape {
                CollisionBox::BoundingBox(bbox) => {
                    let size = bbox.size();
                    let mass = self.mass;
                    let width = size.x;
                    let height = size.y;
                    let depth = size.z;

                    let i_xx = (1.0 / 12.0) * mass * (height * height + depth * depth);
                    let i_yy = (1.0 / 12.0) * mass * (width * width + depth * depth);
                    let i_zz = (1.0 / 12.0) * mass * (width * width + height * height);

                    inertia_tensor.x.x += i_xx;
                    inertia_tensor.y.y += i_yy;
                    inertia_tensor.z.z += i_zz;
                }
                CollisionBox::Sphere(sphere) => {
                    let radius = sphere.radius;
                    let mass = self.mass;

                    let i = (2.0 / 5.0) * mass * radius * radius;

                    inertia_tensor.x.x += i;
                    inertia_tensor.y.y += i;
                    inertia_tensor.z.z += i;
                }
            }
        }

        self.inertia_tensor = inertia_tensor;
        self.inverse_inertia_tensor = self.inertia_tensor.invert().unwrap_or(Matrix3::zero());
    }

    pub fn tangential_velocity(&self, contact_point: Vector3<f32>) -> Vector3<f32> {
        self.angular_velocity.cross(contact_point - self.position)
    }

    pub fn apply_surface_friction(&mut self, contact_normal: Vector3<f32>, contact_point: Vector3<f32>, friction_coefficient: f32) {
        let relative_velocity = self.velocity + self.tangential_velocity(contact_point);

        let tangential_velocity = relative_velocity - contact_normal * relative_velocity.dot(contact_normal);

        let friction_force = -tangential_velocity * friction_coefficient * self.mass;

        self.apply_force(-friction_force);

        let r = contact_point - self.position;

        let torque = r.cross(friction_force);
        self.apply_torque(torque);

        self.velocity *= 0.98;
        self.angular_velocity *= 0.98;
    }
}
