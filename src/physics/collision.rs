use cgmath::{ Vector3, EuclideanSpace, InnerSpace };
use super::collision_box::*;

pub struct Collision {
    pub normal: Vector3<f32>,
    pub penetration_depth: f32,
    pub contact_point: Vector3<f32>,
}

impl Collision {
    pub fn detect(shape_a: &CollisionBox, shape_b: &CollisionBox) -> Option<Collision> {
        match (shape_a, shape_b) {
            (CollisionBox::Sphere(sphere_a), CollisionBox::Sphere(sphere_b)) => {
                let center_distance = (sphere_a.center - sphere_b.center).magnitude();
                let penetration_depth = (sphere_a.radius + sphere_b.radius) - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere_b.center - sphere_a.center).normalize(),
                        penetration_depth,
                        contact_point: (sphere_a.center.to_vec() + sphere_b.center.to_vec()) * 0.5,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::BoundingBox(box_a), CollisionBox::BoundingBox(box_b)) => {
                let x_overlap = (box_a.max.x - box_b.min.x).min(box_b.max.x - box_a.min.x);
                let y_overlap = (box_a.max.y - box_b.min.y).min(box_b.max.y - box_a.min.y);
                let z_overlap = (box_a.max.z - box_b.min.z).min(box_b.max.z - box_a.min.z);

                let mut penetration_depth = x_overlap;
                let mut normal = Vector3::unit_x();

                if y_overlap < penetration_depth {
                    penetration_depth = y_overlap;
                    normal = Vector3::unit_y();
                }

                if z_overlap < penetration_depth {
                    penetration_depth = z_overlap;
                    normal = Vector3::unit_z();
                }

                if (box_b.center() - box_a.center()).dot(normal) < 0.0 {
                    normal = -normal;
                }

                if let Some(overlap_region) = box_a.get_overlap_region(box_b) {
                    Some(Collision {
                        normal,
                        penetration_depth,
                        contact_point: overlap_region.center().to_vec(),
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
                        normal: (sphere.center.to_vec() - closest_point).normalize(),
                        penetration_depth,
                        contact_point: closest_point,
                    })
                } else {
                    None
                }
            }
        }
    }
}
