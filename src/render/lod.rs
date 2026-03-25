use cgmath::InnerSpace;
use cgmath::Vector3;

#[derive(Debug, Clone)]
pub struct LodConfig {
    pub thresholds: Vec<f32>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![100.0, 300.0, 500.0],
        }
    }
}

pub fn select_lod(distance: f32, lod_count: usize, config: &LodConfig) -> usize {
    if lod_count == 0 {
        return 0;
    }

    for (i, threshold) in config.thresholds.iter().enumerate() {
        if distance < *threshold {
            return i.min(lod_count.saturating_sub(1));
        }
    }

    lod_count.saturating_sub(1)
}

pub fn select_lod_by_position(
    view_pos: Vector3<f32>,
    object_pos: Vector3<f32>,
    lod_count: usize,
) -> usize {
    let dist = (view_pos - object_pos).magnitude();
    select_lod(dist, lod_count, &LodConfig::default())
}
