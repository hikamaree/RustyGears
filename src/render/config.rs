pub struct RenderConfig {
    pub near_plane: f32,
    pub far_plane: f32,
    pub fov_degrees: f32,
    pub shadow_frustum_size: f32,
    pub shadow_frustum_near: f32,
    pub shadow_frustum_far: f32,
    pub shadow_cascade_resolution: u32,
    pub clear_color: wgpu::Color,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            near_plane: 0.1,
            far_plane: 1000.0,
            fov_degrees: 45.0,
            shadow_frustum_size: 150.0,
            shadow_frustum_near: 0.1,
            shadow_frustum_far: 400.0,
            shadow_cascade_resolution: 2048,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }
}

impl RenderConfig {
    pub fn projection(&self, width: u32, height: u32) -> crate::Projection {
        crate::Projection::new(
            width,
            height,
            cgmath::Deg(self.fov_degrees),
            self.near_plane,
            self.far_plane,
        )
    }
}
