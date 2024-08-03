use super::body::*;
use cgmath::{ Vector3, InnerSpace, EuclideanSpace };
use super::collision_box::*;


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
                        if let Some(collision) = self.detect_collision(shape_a, shape_b) {
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

    fn detect_collision(&self, shape_a: &CollisionBox, shape_b: &CollisionBox) -> Option<Collision> {
        match (shape_a, shape_b) {
            (CollisionBox::Sphere(sphere_a), CollisionBox::Sphere(sphere_b)) => {
                let center_distance = (sphere_a.center - sphere_b.center).magnitude();
                let penetration_depth = (sphere_a.radius + sphere_b.radius) - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere_b.center - sphere_a.center).normalize(),
                        penetration_depth,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::BoundingBox(box_a), CollisionBox::BoundingBox(box_b)) => {
                let x_overlap = (box_a.max.x - box_b.min.x).min(box_b.max.x - box_a.min.x);
                let y_overlap = (box_a.max.y - box_b.min.y).min(box_b.max.y - box_a.min.y);
                let z_overlap = (box_a.max.z - box_b.min.z).min(box_b.max.z - box_a.min.z);
                let penetration_depth = x_overlap.min(y_overlap).min(z_overlap);
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (box_b.center() - box_a.center()).normalize(),
                        penetration_depth,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::Sphere(sphere), CollisionBox::BoundingBox(bbox)) | (CollisionBox::BoundingBox(bbox), CollisionBox::Sphere(sphere)) => {
                let closest_point = bbox.closest_point(&sphere.center);
                let center_distance = (sphere.center.to_vec() - closest_point).magnitude();
                let penetration_depth = sphere.radius - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere.center.to_vec() - bbox.closest_point(&sphere.center)).normalize(),
                        penetration_depth,
                    })
                } else {
                    None
                }
            }
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

        let k = 10000.0;
        let penetration_force = normal * (collision.penetration_depth * k);

        let friction_coefficient = 0.1;
        let friction_force = relative_velocity * friction_coefficient;

        if body_a.movable {
            let force_a = -penetration_force - friction_force;
            body_a.apply_force(force_a);
        }

        if body_b.movable {
            let force_b = penetration_force + friction_force;
            body_b.apply_force(force_b);
        }

        let percent = 0.1;
        let correction = collision.normal * (collision.penetration_depth / (body_a.mass + body_b.mass)) * percent;

        if body_a.movable {
            let mass = body_a.mass;
            body_a.position -= correction / mass;
        }

        if body_b.movable {
            let mass = body_b.mass;
            body_b.position += correction / mass;
        }
    }
}

struct Collision {
    normal: Vector3<f32>,
    penetration_depth: f32,
}
