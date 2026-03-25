use std::path::Path;
use std::sync::Arc;
use wgpu::Device;
use wgpu::Queue;

pub trait GpuResources: Send + Sync {
    fn device(&self) -> &Device;
    fn queue(&self) -> &Queue;
}

pub trait TextureLoader: Send + Sync {
    fn load_texture(&self, path: &Path, is_normal_map: bool) -> Option<Arc<crate::Texture>>;
    fn create_color_texture(
        &self,
        color: [f32; 4],
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Arc<crate::Texture>;
    fn default_texture(&self) -> Arc<crate::Texture>;
}

/// Simple helper for loading models with just device and queue
pub struct SimpleGpuResources {
    device: Device,
    queue: Queue,
}

impl SimpleGpuResources {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }
}

impl GpuResources for SimpleGpuResources {
    fn device(&self) -> &Device {
        &self.device
    }

    fn queue(&self) -> &Queue {
        &self.queue
    }
}

impl TextureLoader for SimpleGpuResources {
    fn load_texture(&self, path: &Path, is_normal_map: bool) -> Option<Arc<crate::Texture>> {
        let data = std::fs::read(path).ok()?;
        let path_str = path.to_string_lossy();
        crate::Texture::from_bytes(&self.device, &self.queue, &data, &path_str, is_normal_map)
            .ok()
            .map(Arc::new)
    }

    fn create_color_texture(
        &self,
        color: [f32; 4],
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Arc<crate::Texture> {
        Arc::new(crate::Texture::from_color(
            &self.device,
            &self.queue,
            color,
            label,
            is_normal_map,
        ))
    }

    fn default_texture(&self) -> Arc<crate::Texture> {
        crate::Texture::default(&self.device, &self.queue)
    }
}
