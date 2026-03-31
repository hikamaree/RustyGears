use crate::render::lod::select_lod_by_position;
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

pub fn select_model_lod(
    render_obj: &crate::RenderObject,
    object_pos: Vector3<f32>,
    view_pos: Vector3<f32>,
) -> Option<(crate::Model3d, usize)> {
    if render_obj.lods.is_empty() {
        return None;
    }
    let lod_index = select_lod_by_position(view_pos, object_pos, render_obj.lods.len());
    Some((render_obj.lods[lod_index].clone(), lod_index))
}

pub fn get_mesh_world_bounds(
    mesh: &crate::model::Mesh,
    model_mat: &Matrix4<f32>,
) -> (Vector3<f32>, f32) {
    let scale = calculate_model_scale(model_mat);
    transform_bounding_sphere(
        mesh.data.bounding_sphere.center,
        mesh.data.bounding_sphere.radius * scale,
        model_mat,
    )
}
