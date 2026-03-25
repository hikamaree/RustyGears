use crate::math::InnerSpace;
use crate::math::Vector3;
use crate::ModelRenderData;

pub trait SortStrategy: Send + Sync {
    fn sort(&self, items: &mut [ModelRenderData], view_pos: Vector3<f32>);
}

pub struct NoSort;

impl SortStrategy for NoSort {
    fn sort(&self, _items: &mut [ModelRenderData], _view_pos: Vector3<f32>) {}
}

pub struct DepthSortBackToFront;

impl SortStrategy for DepthSortBackToFront {
    fn sort(&self, items: &mut [ModelRenderData], view_pos: Vector3<f32>) {
        items.sort_by(|a, b| {
            let center_a = get_batch_center(a, view_pos);
            let center_b = get_batch_center(b, view_pos);
            center_b
                .partial_cmp(&center_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

pub struct DepthSortFrontToBack;

impl SortStrategy for DepthSortFrontToBack {
    fn sort(&self, items: &mut [ModelRenderData], view_pos: Vector3<f32>) {
        items.sort_by(|a, b| {
            let center_a = get_batch_center(a, view_pos);
            let center_b = get_batch_center(b, view_pos);
            center_a
                .partial_cmp(&center_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

fn get_batch_center(batch: &ModelRenderData, view_pos: Vector3<f32>) -> f32 {
    if batch.instance_data.is_empty() {
        return 0.0;
    }
    let first = &batch.instance_data[0];
    let pos = Vector3::new(first.model[3][0], first.model[3][1], first.model[3][2]);
    (pos - view_pos).magnitude2()
}
