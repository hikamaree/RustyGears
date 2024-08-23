use super::body::*;
use cgmath::{
    Vector3,
    InnerSpace
};
use super::collision::*;

use rayon::prelude::*;

pub struct PhysicsWorld {
    pub(crate) bodies: Vec<BodyRef>,
    pub(crate) gravity: Vector3<f32>,
    pub(crate) delta_time: f32,
    pub(crate) bounds: f32,
}

impl PhysicsWorld {
    pub fn new(gravity: Vector3<f32>) -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity,
            delta_time: 1.0 / 1000.0,
            bounds: 1000.0
        }
    }

    pub fn set_gravity(&mut self, gravity: Vector3<f32>) {
        self.gravity = gravity;
    }

    pub fn set_refresh_frequency(&mut self, frequency: f32) {
        self.delta_time = 1.0 / frequency;
    }

    pub fn add_body(&mut self, body: BodyRef) {
        self.bodies.push(body);
    }

    pub fn update(&mut self, mut dt: f32) {
        while dt > 0.0 {
            for body_ref in &mut self.bodies {

                let mut body = body_ref.lock().unwrap();

                if body.gravity && body.movable {
                    let mass = body.mass;
                    body.apply_force(self.gravity * mass);
                }

                body.update(self.delta_time);
            }
            self.handle_collisions();
            dt -= self.delta_time;
        }
        self.filter_bodies();
    }

    fn handle_collisions(&mut self) {
        let body_count = self.bodies.len();

        let collisions: Vec<(usize, usize, Collision)> = (0..body_count)
            .into_par_iter()
            .flat_map(|i| {
                (i + 1..body_count)
                    .into_par_iter()
                    .filter_map(|j| self.get_collision(i, j).map(|collision| (i, j, collision)))
                    .collect::<Vec<_>>()
            })
        .collect();

        collisions.into_par_iter().for_each(|(i, j, collision)| {
            self.resolve_collision(i, j, collision);
        });
    }

    fn get_collision(&self, i: usize, j: usize) -> Option<Collision> {
        let body_a = self.bodies[i].lock().unwrap();
        let body_b = self.bodies[j].lock().unwrap();

        if !body_a.movable && !body_b.movable {
            return None;
        }
        for shape_a in body_a.collision_box.iter() {
            for shape_b in body_b.collision_box.iter() {
                if let Some(collision) = Collision::detect(&shape_a, &shape_b, &body_a.rotation, &body_b.rotation) {
                    return Some(collision);
                }
            }
        }
        None
    }

    fn resolve_collision(&self, i: usize, j: usize, collision: Collision) {
        let mut body_a = self.bodies[i].lock().unwrap();
        let mut body_b = self.bodies[j].lock().unwrap();

        let r_a = collision.contact_point - body_a.position;
        let r_b = collision.contact_point - body_b.position;
        let relative_velocity = (body_b.velocity + r_b.cross(body_b.angular_velocity)) - (body_a.velocity + r_a.cross(body_a.angular_velocity));
        let normal = collision.normal;

        let bounciness = (body_a.bounciness + body_b.bounciness) / 2.0;
        let impulse_magnitude = -(1.0 + bounciness) * relative_velocity.dot(normal) / (1.0 / body_a.mass + 1.0 / body_b.mass);
        let impulse = impulse_magnitude * normal;

        let friction_coefficient = (body_a.friction_coefficient * body_b.friction_coefficient).sqrt() * 10.0;

        let correction = collision.normal * (collision.overlap / (body_a.mass + body_b.mass)) / 2.0;

        if body_a.movable {
            let mass = body_a.mass;
            body_a.velocity -= impulse / mass;


            let torque_a = -r_a.cross(impulse / self.delta_time);
            body_a.apply_torque(torque_a);

            let gravity_torque = -r_a.cross(self.gravity * mass);
            body_a.apply_torque(gravity_torque);

            body_a.apply_surface_friction(normal, collision.contact_point, friction_coefficient, self.delta_time);

            body_a.correction = correction * body_b.mass;
        }

        if body_b.movable {
            let mass = body_b.mass;
            body_b.velocity += impulse / mass;

            let torque_b = -r_b.cross(impulse / self.delta_time);
            body_a.apply_torque(torque_b);

            let gravity_torque = -r_b.cross(self.gravity * mass);
            body_b.apply_torque(gravity_torque);

            body_b.apply_surface_friction(normal, collision.contact_point, friction_coefficient, self.delta_time);

            body_b.correction = correction * body_a.mass;
        }
    }

    fn filter_bodies(&mut self) {
        let mut new_bodies = Vec::new();
        for body_ref in self.bodies.drain(..) {
            let body = body_ref.lock().unwrap();
            if body.position.magnitude() <= self.bounds {
                new_bodies.push(body_ref.clone());
            }
        }
        self.bodies = new_bodies;
    }
}
