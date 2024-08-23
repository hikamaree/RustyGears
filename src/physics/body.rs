use cgmath::{
    Vector3,
    Matrix3,
    Point3,
    Quaternion,
    Rotation3,
    Zero,
    One,
    Rad,
    SquareMatrix,
    InnerSpace,
    EuclideanSpace,
    Array
};
use super::collision_box::*;
use crate::model::*;

use std::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct RigidBody {
    pub position: Vector3<f32>,
    pub correction: Vector3<f32>,
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

pub type BodyRef = Arc<Mutex<RigidBody>>;

impl RigidBody {
    pub fn new(collision_box: Vec<CollisionBox>) -> BodyRef {
        let mut body: RigidBody = RigidBody {
            position: Vector3::zero(),
            correction: Vector3::zero(),
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            mass: 1.0,
            forces: Vector3::zero(),
            gravity: true,
            movable: true,
            collision_box,
            rotation: Quaternion::one(),
            angular_velocity: Vector3::zero(),
            angular_acceleration: Vector3::zero(),
            torque: Vector3::zero(),
            inertia_tensor: Matrix3::identity(),
            inverse_inertia_tensor: Matrix3::identity(),
            bounciness: 0.5,
            friction_coefficient: 1.0,
        };

        body.set_position(body.position);
        body.update_inertia_tensor();
        Arc::new(Mutex::new(body))
    }

    pub fn with_bboxes(model: &Model) -> BodyRef {
        let collision_box = model.get_meshes().iter().map(|mesh| {
            CollisionBox::BoundingBox(mesh.calculate_bounding_box())
        }).collect();

        RigidBody::new(collision_box)
    }

    pub fn with_spheres(model: &Model) -> BodyRef {
        let collision_box = model.get_meshes().iter().map(|mesh| {
            CollisionBox::Sphere(mesh.calculate_sphere())
        }).collect();

        RigidBody::new(collision_box)
    }

    pub fn with_single_bbox(model: &Model) -> BodyRef {
        let mut min = Vector3::from_value(f32::MAX);
        let mut max = Vector3::from_value(f32::MIN);

        for mesh in model.get_meshes() {
            let bounding_box = mesh.calculate_bounding_box();

            min.x = min.x.min(bounding_box.min.x);
            min.y = min.y.min(bounding_box.min.y);
            min.z = min.z.min(bounding_box.min.z);

            max.x = max.x.max(bounding_box.max.x);
            max.y = max.y.max(bounding_box.max.y);
            max.z = max.z.max(bounding_box.max.z);
        }

        let combined_bounding_box = BoundingBox {
            min: Point3::from_vec(min),
            max: Point3::from_vec(max)
        };

        let collision_box = CollisionBox::BoundingBox(combined_bounding_box);
        RigidBody::new(vec![collision_box])
    }


    pub fn with_single_sphere(model: &Model) -> BodyRef {
        let mut center = Vector3::zero();
        let mut total_vertices = 0;

        for mesh in model.get_meshes() {
            let mesh_center = mesh.vertices.iter().fold(Vector3::zero(), |acc, v| acc + v.position);
            center += mesh_center;
            total_vertices += mesh.vertices.len();
        }

        center /= total_vertices as f32;

        let mut radius: f32 = 0.0;
        for mesh in model.get_meshes() {
            for vertex in &mesh.vertices {
                radius = radius.max((vertex.position - center).magnitude());
            }
        }

        let sphere = Sphere {
            center: Point3::from_vec(center),
            radius,
        };

        let collision_box = CollisionBox::Sphere(sphere);
        RigidBody::new(vec![collision_box])
    }

    pub fn set_position(&mut self, positon: Vector3<f32>) {
        let delta = positon - self.position;
        self.position = positon;
        for shape in &mut self.collision_box {
            match shape {
                CollisionBox::BoundingBox(bbox) => {
                    bbox.min += delta;
                    bbox.max += delta;
                }
                CollisionBox::Sphere(sphere) => {
                    sphere.center += delta;
                }
            }
        }
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
        self.update_inertia_tensor();
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

    pub fn set_gravity(&mut self, gravity: bool) -> Self{
        self.gravity = gravity;
        self.clone()
    }

    pub fn update(&mut self, dt: f32) {
        if !self.movable { return; }
        self.acceleration = self.forces / self.mass;
        self.velocity += self.acceleration * dt;

        self.angular_acceleration = self.inverse_inertia_tensor * self.torque;
        self.angular_velocity += self.angular_acceleration * dt;

        let delta_rotation = Quaternion::from_angle_x(Rad(self.angular_velocity.x * dt)) *
            Quaternion::from_angle_y(Rad(self.angular_velocity.y * dt)) *
            Quaternion::from_angle_z(Rad(self.angular_velocity.z * dt));

        self.set_position(self.position + self.velocity * dt + self.correction);
        self.rotation = (delta_rotation * self.rotation).normalize();

        self.correction = Vector3::zero();
        self.forces = Vector3::zero();
        self.torque = Vector3::zero();
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

    pub fn apply_surface_friction(&mut self, contact_normal: Vector3<f32>, contact_point: Vector3<f32>, friction_coefficient: f32, dumping: f32) {
        let relative_velocity = self.velocity + self.tangential_velocity(contact_point);

        let tangential_velocity = relative_velocity - contact_normal * relative_velocity.dot(contact_normal);

        let friction_force =
            if tangential_velocity.magnitude() < 0.1 {
                -tangential_velocity * friction_coefficient * self.mass
            } else {
                -tangential_velocity.normalize() * friction_coefficient * self.mass
            };

        self.apply_force(friction_force);

        let r = contact_point - self.position;

        let torque = r.cross(friction_force);
        self.apply_torque(torque);

        self.velocity *= 1.0 - dumping;
        self.angular_velocity *= 1.0 - dumping;
    }
}
