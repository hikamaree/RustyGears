use super::body::*;
use cgmath::{
    Vector3,
//    Point3,
//    EuclideanSpace,
    InnerSpace
};
use super::collision::*;
//use super::octree::*;

use rayon::prelude::*;

pub struct PhysicsWorld {
    pub bodies: Vec<BodyRef>,
    pub gravity: Vector3<f32>,
    delta_time: f32,
}

impl PhysicsWorld {
    pub fn new(gravity: Vector3<f32>) -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity,
            delta_time: 1.0 / 1000.0,
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

                let mut body = body_ref.lock().unwrap();//borrow_mut();

                if body.gravity && body.movable {
                    let mass = body.mass;
                    body.apply_force(self.gravity * mass);
                }

                body.update(self.delta_time);
            }
            self.handle_collisions();
            dt -= self.delta_time;
        }
    }

    /*pub fn handle_collisions(&mut self) {
        let mut collisions = Vec::new();
        let body_count = self.bodies.len();

        for i in 0..body_count {
            for j in (i + 1)..body_count {
                if let Some(collision) = self.get_colllision(i, j) {
                    collisions.push((i, j, collision));
                }
            }
        }

        for (i, j, collision) in collisions {
            self.resolve_collision(i, j, collision);
        }
    }*/

    pub fn handle_collisions(&mut self) {
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

    /*pub fn handle_collisions(&mut self) {
      let mut collisions = Vec::new();

      let world_boundary = BoundingBox::new(Point3::new(-100.0, -100.0, -100.0), Point3::new(100.0, 100.0, 100.0)); // postavi odgovarajuće granice
      let mut octree = Octree::new(world_boundary, 4);

      for (i, body) in self.bodies.iter().enumerate() {
      octree.insert(i, Point3::from_vec(body.borrow().position));
      }

      for i in 0..self.bodies.len() {
      let body_a = &self.bodies[i];
      let mut potential_collisions = Vec::new();

      octree.query(/*&body_a.bounding_box*/, &mut potential_collisions);

    for &j in &potential_collisions {
        if i != j {
            if let Some(collision) = self.get_colllision(i, j) {
                collisions.push((i, j, collision));
            }
        }
    }
}

for (i, j, collision) in collisions {
    self.resolve_collision(i, j, collision);
}
}*/

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

    if !body_a.movable && !body_b.movable {
        return;
    }

    let r_a = collision.contact_point - body_a.position;
    let r_b = collision.contact_point - body_b.position;
    let relative_velocity = (body_b.velocity + r_b.cross(body_b.angular_velocity)) - (body_a.velocity + r_a.cross(body_a.angular_velocity));
    let normal = collision.normal;

    let bounciness = (body_a.bounciness + body_b.bounciness) / 2.0;
    let impulse_magnitude = -(1.0 + bounciness) * relative_velocity.dot(normal) / (1.0 / body_a.mass + 1.0 / body_b.mass);
    let impulse = impulse_magnitude * normal;

    let friction_coefficient = (body_a.friction_coefficient * body_b.friction_coefficient).sqrt() * 10.0;

    let percent = 0.5;
    let correction = collision.normal * (collision.overlap / (body_a.mass + body_b.mass)) * percent;

    if body_a.movable {
        let mass = body_a.mass;
        body_a.velocity -= impulse / mass;


        let torque_a = normal.cross(impulse * body_b.mass);
        body_a.apply_torque(torque_a);

        let gravity_torque = -r_a.cross(self.gravity * mass);
        body_a.apply_torque(gravity_torque);

        body_a.apply_surface_friction(normal, collision.contact_point, friction_coefficient, self.delta_time);

        body_a.correction = -correction * body_b.mass;
    }

    if body_b.movable {
        let mass = body_b.mass;
        body_b.velocity += impulse / mass;

        let torque_b = normal.cross(-impulse * body_a.mass);
        body_a.apply_torque(torque_b);

        let gravity_torque = -r_b.cross(self.gravity * mass);
        body_b.apply_torque(gravity_torque);

        body_b.apply_surface_friction(normal, collision.contact_point, friction_coefficient, self.delta_time);

        body_b.correction = correction * body_a.mass;
    }
}
}
