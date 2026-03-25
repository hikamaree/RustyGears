use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OutputHandle(pub usize);

#[derive(Clone)]
pub enum TargetSize {
    Screen,
    Fixed(u32, u32),
    Fraction(f32),
}

impl PartialEq for TargetSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TargetSize::Screen, TargetSize::Screen) => true,
            (TargetSize::Fixed(a_w, a_h), TargetSize::Fixed(b_w, b_h)) => a_w == b_w && a_h == b_h,
            (TargetSize::Fraction(a), TargetSize::Fraction(b)) => (a - b).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

impl Eq for TargetSize {}

impl std::hash::Hash for TargetSize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TargetSize::Screen => 0u8.hash(state),
            TargetSize::Fixed(w, h) => {
                1u8.hash(state);
                w.hash(state);
                h.hash(state);
            }
            TargetSize::Fraction(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
        }
    }
}

impl TargetSize {
    fn resolve(&self, screen_width: u32, screen_height: u32) -> (u32, u32) {
        match self {
            TargetSize::Screen => (screen_width, screen_height),
            TargetSize::Fixed(w, h) => (*w, *h),
            TargetSize::Fraction(f) => (
                (screen_width as f32 * f) as u32,
                (screen_height as f32 * f) as u32,
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TargetDescriptor {
    pub format: wgpu::TextureFormat,
    pub size: TargetSize,
    pub sample_count: u32,
    pub usage: wgpu::TextureUsages,
}

pub struct RenderTarget {
    pub descriptor: TargetDescriptor,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        descriptor: &TargetDescriptor,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: descriptor.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: descriptor.format,
            usage: descriptor.usage,
            view_formats: &[],
        });

        Self {
            descriptor: descriptor.clone(),
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            width,
            height,
        }
    }
}

pub struct RenderTargetPool {
    targets: HashMap<usize, RenderTarget>,
    next_handle: usize,
    cache: HashMap<(wgpu::TextureFormat, u32, u32, u32), Vec<usize>>,
    cached_screen_size: (u32, u32),
}

impl RenderTargetPool {
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
            next_handle: 0,
            cache: HashMap::new(),
            cached_screen_size: (0, 0),
        }
    }

    fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        descriptor: &TargetDescriptor,
        screen_width: u32,
        screen_height: u32,
    ) -> usize {
        let (width, height) = descriptor.size.resolve(screen_width, screen_height);
        let cache_key = (descriptor.format, width, height, descriptor.sample_count);

        if let Some(handle) = self.cache.get(&cache_key).and_then(|v| v.first().cloned()) {
            if let Some(target) = self.targets.get_mut(&handle) {
                if target.width == width && target.height == height {
                    return handle;
                }
            }
        }

        let handle = self.next_handle;
        self.next_handle += 1;

        let target = RenderTarget::new(device, descriptor, width, height, "render_target");
        self.targets.insert(handle, target);

        self.cache
            .entry(cache_key)
            .or_insert_with(Vec::new)
            .push(handle);

        handle
    }

    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        descriptor: &TargetDescriptor,
        screen_width: u32,
        screen_height: u32,
    ) -> OutputHandle {
        if self.cached_screen_size != (screen_width, screen_height) {
            self.cached_screen_size = (screen_width, screen_height);
            self.cache.clear();
        }

        let handle = self.get_or_create(device, descriptor, screen_width, screen_height);
        OutputHandle(handle)
    }

    pub fn get(&self, handle: OutputHandle) -> Option<&RenderTarget> {
        self.targets.get(&handle.0)
    }

    pub fn get_view(&self, handle: OutputHandle) -> Option<&wgpu::TextureView> {
        self.targets.get(&handle.0).map(|t| &t.view)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.cached_screen_size = (0, 0);
    }
}

impl Default for RenderTargetPool {
    fn default() -> Self {
        Self::new()
    }
}
