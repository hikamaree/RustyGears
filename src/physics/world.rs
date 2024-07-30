use super::body::*;
use cgmath::{ Vector3, InnerSpace, EuclideanSpace };
use super::collision_box::*;


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
        self.bodies[0].ignore_gravity();
        self.bodies[2].ignore_gravity();
        self.bodies[3].ignore_gravity();
        for body in &mut self.bodies {

            if body.gravity {
                body.apply_force(self.gravity * body.mass);
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
                for shape_a in &self.bodies[i].collision_box {
                    for shape_b in &self.bodies[j].collision_box {
                        if let Some(collision) = self.detect_collision(shape_a, shape_b) {
                            collisions.push((i, j, collision));
                        }
                    }
                }
            }
        }

        for (i, j, collision) in collisions {
            println!("Collision {}, {}", i, j);
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
        let normal = (self.bodies[j].position - self.bodies[i].position).normalize();
        let relative_velocity = self.bodies[j].velocity - self.bodies[i].velocity;
        let impulse = -(1.0 + 0.2) * relative_velocity.dot(normal);

        let v1 = normal * impulse / self.bodies[i].mass;
        let v2 = normal * impulse / self.bodies[j].mass;
        self.bodies[i].velocity -= v1;
        self.bodies[j].velocity += v2;

        let correction = collision.normal * (collision.penetration_depth / 2.0);
        self.bodies[i].position -= correction;
        self.bodies[j].position += correction;
    }
}

struct Collision {
    normal: Vector3<f32>,
    penetration_depth: f32,
}
