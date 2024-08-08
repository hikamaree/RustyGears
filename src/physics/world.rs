use super::body::*;
use cgmath::{ Vector3, InnerSpace };
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
                        if let Some(collision) = Collision::detect(shape_a, shape_b) {
                            collisions.push((i, j, collision));
                        }
                    }
                }
            }
        }

        for (i, j, collision) in collisions {
            self.resolve_collision(i, j, collision);
        }
    }

    fn resolve_collision(&mut self, i: usize, j: usize, collision: Collision) {
        let mut body_a = self.bodies[i].borrow_mut();
        let mut body_b = self.bodies[j].borrow_mut();

        if !body_a.movable && !body_b.movable {
            return;
        }

        let normal = collision.normal.normalize();
        let relative_velocity = body_b.velocity - body_a.velocity;
        let velocity_along_normal = relative_velocity.dot(normal);

        if velocity_along_normal > 0.0 {
            return;
        }

        let restitution = 0.3; // 0.0 - 1.0
        let j = (-(1.0 + restitution) * velocity_along_normal) / (1.0 / body_a.mass + 1.0 / body_b.mass);
        let impulse = j * normal;

        let k = 10000.0;
        let penetration_force = normal * (collision.penetration_depth * k);

        let friction_coefficient = 0.5;

        let percent = 0.1;
        let correction = collision.normal * (collision.penetration_depth / (body_a.mass + body_b.mass)) * percent;

        if body_a.movable {
            let mass = body_a.mass;

            body_a.velocity -= impulse / mass;

            let force_a = -penetration_force;
            body_a.apply_force(force_a);

            let r_a = collision.contact_point - body_a.position;
            let torque_a = -r_a.cross(body_a.velocity * body_a.mass + body_b.velocity * body_b.mass);
            body_a.apply_torque(torque_a);

            body_a.apply_surface_friction(normal, collision.contact_point, friction_coefficient);

            let rotational_friction = -body_a.angular_velocity * friction_coefficient * mass;
            body_a.apply_torque(rotational_friction);

            body_a.position -= correction / mass;
        }

        if body_b.movable {
            let mass = body_b.mass;

            body_b.velocity += impulse / mass;

            let force_b = penetration_force;
            body_b.apply_force(force_b);

            let r_b = collision.contact_point - body_b.position;
            let torque_b = -r_b.cross(body_a.velocity * body_a.mass + body_b.velocity * body_b.mass);
            body_b.apply_torque(torque_b);

            body_b.apply_surface_friction(normal, collision.contact_point ,friction_coefficient);

            let rotational_friction = -body_b.angular_velocity * friction_coefficient * mass;
            body_b.apply_torque(rotational_friction);

            body_b.position += correction / mass;
        }
    }
}
