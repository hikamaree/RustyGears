use super::body::*;
use cgmath::{
    Vector3,
    InnerSpace
};
use super::collision::*;

pub struct PhysicsWorld {
    pub bodies: Vec<BodyRef>,
    pub gravity: Vector3<f32>,
}

impl PhysicsWorld {
    pub fn new(gravity: Vector3<f32>) -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity,
        }
    }

    pub fn add_body(&mut self, body: BodyRef) {
        self.bodies.push(body);
    }

    pub fn update(&mut self, dt: f32) {
        for body_ref in &mut self.bodies {

            let mut body = body_ref.borrow_mut();

            if body.gravity && body.movable {
                let mass = body.mass;
                body.apply_force(self.gravity * mass);
            }

            body.update(dt);
        }
        self.handle_collisions();
    }

    pub fn handle_collisions(&mut self) {
        let mut collisions = Vec::new();
        let body_count = self.bodies.len();

        for i in 0..body_count {
            for j in (i + 1)..body_count {
                for shape_a in &self.bodies[i].borrow().collision_box {
                    for shape_b in &self.bodies[j].borrow().collision_box {
                        if let Some(collision) = Collision::detect(shape_a, shape_b, self.bodies[i].borrow().rotation, self.bodies[j].borrow().rotation) {
                            collisions.push((i, j, collision));
                        }
                    }
                }
            }
        }

        for (i, j, collision) in collisions {
            self.resolve_collision(i as u32, j as u32, collision);
        }
    }

    fn resolve_collision(&mut self, i: u32, j: u32, collision: Collision) {
        let mut body_a = self.bodies[i as usize].borrow_mut();
        let mut body_b = self.bodies[j as usize].borrow_mut();

        if !body_a.movable && !body_b.movable {
            return;
        }

        let normal = collision.normal.normalize();
        let relative_velocity = body_b.velocity - body_a.velocity;
        let velocity_along_normal = relative_velocity.dot(normal);

        let bounciness = (body_a.bounciness + body_b.bounciness) / 2.0;
        let j = (-(1.0 + bounciness) * velocity_along_normal) / (1.0 / body_a.mass + 1.0 / body_b.mass);
        let impulse = j * normal;

        let k = 10000.0;
        let force = normal * (collision.overlap * k);

        let friction_coefficient = (body_a.friction_coefficient * body_b.friction_coefficient).sqrt();

        let percent = 0.2;
        let correction = collision.normal * (collision.overlap / (body_a.mass + body_b.mass)) * percent;

        if body_a.movable {
            let mass = body_a.mass;

            body_a.velocity -= impulse / mass;

            body_a.apply_force(-force);

            let r_a = collision.contact_point - body_a.position;
            let torque_a = -r_a.cross(body_a.velocity * body_a.mass + body_b.velocity * body_b.mass);
            body_a.apply_torque(torque_a);


            let gravity_torque = r_a.cross(self.gravity * body_a.mass);
            body_a.apply_torque(-gravity_torque);

            body_a.apply_surface_friction(normal, collision.contact_point, friction_coefficient);
            body_a.position -= correction * mass;
        }

        if body_b.movable {
            let mass = body_b.mass;

            body_b.velocity += impulse / mass;

            body_b.apply_force(force);

            let r_b = collision.contact_point - body_b.position;
            let torque_b = -r_b.cross(body_a.velocity * body_a.mass + body_b.velocity * body_b.mass);
            body_b.apply_torque(torque_b);

            let gravity_torque = r_b.cross(self.gravity * body_b.mass);
            body_b.apply_torque(-gravity_torque);

            body_b.apply_surface_friction(normal, collision.contact_point ,friction_coefficient);
            body_b.position += correction * body_a.mass;
        }
    }
}
