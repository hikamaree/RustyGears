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
            (CollisionBox::BoundingBox(aabb_a), CollisionBox::BoundingBox(aabb_b)) => {
                let x_overlap = (aabb_a.max.x - aabb_b.min.x).min(aabb_b.max.x - aabb_a.min.x);
                let y_overlap = (aabb_a.max.y - aabb_b.min.y).min(aabb_b.max.y - aabb_a.min.y);
                let z_overlap = (aabb_a.max.z - aabb_b.min.z).min(aabb_b.max.z - aabb_a.min.z);
                let penetration_depth = x_overlap.min(y_overlap).min(z_overlap);
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (aabb_b.center() - aabb_a.center()).normalize(),
                        penetration_depth,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::Sphere(sphere), CollisionBox::BoundingBox(aabb)) | (CollisionBox::BoundingBox(aabb), CollisionBox::Sphere(sphere)) => {
                let closest_point = aabb.closest_point(&sphere.center);
                let center_distance = (sphere.center.to_vec() - closest_point).magnitude();
                let penetration_depth = sphere.radius - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere.center.to_vec() - aabb.closest_point(&sphere.center)).normalize(),
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

        let normal = (body_b.position - body_a.position).normalize();
        let relative_velocity = body_b.velocity - body_a.velocity;
        let impulse = relative_velocity.dot(normal) * (body_a.mass + body_b.mass) * 0.1;

        let v1 = normal * impulse / body_a.mass;
        let v2 = normal * impulse / body_b.mass;

        if !body_a.movable {
            body_b.velocity = -v2;
            body_b.position -= collision.normal * collision.penetration_depth;
        } else if !body_b.movable {
            body_a.velocity = v1;
            body_a.position -= collision.normal * collision.penetration_depth;
        } else {
            body_a.velocity = v1;
            body_b.velocity = -v2;
            body_a.position += collision.normal * (collision.penetration_depth / 2.0);
            body_b.position -= collision.normal * (collision.penetration_depth / 2.0);
        }
    }
}

struct Collision {
    normal: Vector3<f32>,
    penetration_depth: f32,
}
