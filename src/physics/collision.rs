use cgmath::{
    Vector3,
    EuclideanSpace,
    InnerSpace,
    Quaternion,
    Zero,
    Matrix3,
    Point3,
};
use super::collision_box::*;

pub struct Collision {
    pub normal: Vector3<f32>,
    pub overlap: f32,
    pub contact_point: Vector3<f32>,
}

impl Collision {
    pub fn detect(shape_a: &CollisionBox, shape_b: &CollisionBox, rotation_a: Quaternion<f32>, rotation_b: Quaternion<f32>) -> Option<Collision> {
        match (shape_a, shape_b) {
            (CollisionBox::Sphere(sphere_a), CollisionBox::Sphere(sphere_b)) => {
                let center_distance = (sphere_a.center - sphere_b.center).magnitude();
                let penetration_depth = (sphere_a.radius + sphere_b.radius) - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere_b.center - sphere_a.center).normalize(),
                        overlap: penetration_depth,
                        contact_point: (sphere_a.center.to_vec() + sphere_b.center.to_vec()) * 0.5,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::BoundingBox(box_a), CollisionBox::BoundingBox(box_b)) => {
                let rot_box_a = Matrix3::from(rotation_a);
                let rot_box_b = Matrix3::from(rotation_b);

                let half_box_a = box_a.size() / 2.0;
                let half_box_b = box_b.size() / 2.0;

                let diff = box_b.center() - box_a.center();

                let axes = [
                    rot_box_a.x, rot_box_a.y, rot_box_a.z,
                    rot_box_b.x, rot_box_b.y, rot_box_b.z,
                    rot_box_a.x.cross(rot_box_b.x),
                    rot_box_a.x.cross(rot_box_b.y),
                    rot_box_a.x.cross(rot_box_b.z),
                    rot_box_a.y.cross(rot_box_b.x),
                    rot_box_a.y.cross(rot_box_b.y),
                    rot_box_a.y.cross(rot_box_b.z),
                    rot_box_a.z.cross(rot_box_b.x),
                    rot_box_a.z.cross(rot_box_b.y),
                    rot_box_a.z.cross(rot_box_b.z),
                ];

                let mut min_overlap = f32::MAX;
                let mut normal = Vector3::zero();

                for axis in axes.iter() {
                    if axis.magnitude2() < 1e-6 {
                        continue;
                    }

                    let axis = axis.normalize();
                    let projection_box_a = half_box_a.x * (rot_box_a.x.dot(axis)).abs()
                        + half_box_a.y * (rot_box_a.y.dot(axis)).abs()
                        + half_box_a.z * (rot_box_a.z.dot(axis)).abs();

                    let projection_box_b = half_box_b.x * (rot_box_b.x.dot(axis)).abs()
                        + half_box_b.y * (rot_box_b.y.dot(axis)).abs()
                        + half_box_b.z * (rot_box_b.z.dot(axis)).abs();

                    let distance = diff.dot(axis).abs();
                    let overlap = projection_box_a + projection_box_b - distance;

                    if overlap <= 0.0 {
                        return None;
                    }

                    if overlap < min_overlap {
                        min_overlap = overlap;
                        normal = axis.normalize();
                    }
                }

                let corners_a = box_a.rotated_points(&rotation_a);
                let corners_b = box_b.rotated_points(&rotation_b);

                let mut closest_distance = f32::INFINITY;
                let mut contact_point = Point3::new(0.0, 0.0, 0.0);

                for corner_a in corners_a {
                    let point_on_b = box_b.closest_point(&Point3::from_vec(corner_a));
                    let distance = (corner_a - point_on_b).magnitude();
                    if distance < closest_distance {
                        closest_distance = distance;
                        contact_point = Point3::from_vec(point_on_b);
                    }
                }

                for corner_b in corners_b {
                    let point_on_a = box_a.closest_point(&Point3::from_vec(corner_b));
                    let distance = (corner_b - point_on_a).magnitude();
                    if distance < closest_distance {
                        closest_distance = distance;
                        contact_point = Point3::from_vec(point_on_a);
                    }
                }
                Some(Collision {
                    normal,
                    overlap: min_overlap,
                    contact_point: contact_point.to_vec(),
                })
            }
            (CollisionBox::Sphere(sphere), CollisionBox::BoundingBox(bbox)) => {
                let closest_point = bbox.closest_point(&sphere.center);
                let center_distance = (sphere.center.to_vec() - closest_point).magnitude();
                let penetration_depth = sphere.radius - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: -(sphere.center.to_vec() - closest_point).normalize(),
                        overlap: penetration_depth,
                        contact_point: closest_point,
                    })
                } else {
                    None
                }
            }
            (CollisionBox::BoundingBox(bbox), CollisionBox::Sphere(sphere)) => {
                let closest_point = bbox.closest_point(&sphere.center);
                let center_distance = (sphere.center.to_vec() - closest_point).magnitude();
                let penetration_depth = sphere.radius - center_distance;
                if penetration_depth > 0.0 {
                    Some(Collision {
                        normal: (sphere.center.to_vec() - closest_point).normalize(),
                        overlap: penetration_depth,
                        contact_point: closest_point,
                    })
                } else {
                    None
                }
            }
        }
    }
}
