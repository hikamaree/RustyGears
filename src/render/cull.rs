use cgmath::{InnerSpace, Matrix, Matrix4, Vector3, Vector4};

pub fn calculate_model_scale(model_mat: &Matrix4<f32>) -> f32 {
    model_mat
        .x
        .truncate()
        .magnitude()
        .max(model_mat.y.truncate().magnitude())
        .max(model_mat.z.truncate().magnitude())
}

pub fn transform_bounding_sphere(
    center: Vector3<f32>,
    radius: f32,
    model_matrix: &Matrix4<f32>,
) -> (Vector3<f32>, f32) {
    let point: Vector4<f32> = *model_matrix * Vector4::new(center.x, center.y, center.z, 1.0);
    let world_center = Vector3::new(point.x, point.y, point.z);
    let world_radius = radius * calculate_model_scale(model_matrix);
    (world_center, world_radius)
}

pub fn compute_frustum_planes(light_view_proj: &Matrix4<f32>) -> [Vector4<f32>; 6] {
    let m = *light_view_proj;

    let planes = [
        m.row(3) + m.row(0),
        m.row(3) - m.row(0),
        m.row(3) + m.row(1),
        m.row(3) - m.row(1),
        m.row(3) + m.row(2),
        m.row(3) - m.row(2),
    ];

    planes.map(|plane| {
        let normal = Vector3::new(plane.x, plane.y, plane.z);
        let len = normal.magnitude();
        if len > 0.0001 {
            plane / len
        } else {
            plane
        }
    })
}

pub fn is_in_frustum(
    frustum_planes: &[Vector4<f32>; 6],
    center: Vector3<f32>,
    radius: f32,
) -> bool {
    for plane in frustum_planes {
        if plane.dot(Vector4::new(center.x, center.y, center.z, 1.0)) < -radius {
            return false;
        }
    }
    true
}
