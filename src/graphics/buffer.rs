pub const DEFAULT_INSTANCE_BUFFER_SIZE: usize = 32 * 1024 * 1024;
pub const DEFAULT_CAMERA_BUFFER_SIZE: usize = 80;

pub enum BufferStrategy {
    Single,
    Double,
    Triple,
}

pub struct Buffer {
    buffers: Vec<wgpu::Buffer>,
    current_idx: usize,
    strategy: BufferStrategy,
    label: String,
}

impl Buffer {
    pub fn new(
        device: &wgpu::Device,
        size: usize,
        usage: wgpu::BufferUsages,
        strategy: BufferStrategy,
        label: &str,
    ) -> Self {
        let count = match strategy {
            BufferStrategy::Single => 1,
            BufferStrategy::Double => 2,
            BufferStrategy::Triple => 3,
        };
        
        let buffers = (0..count)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{}_buffer_{}", label, i)),
                    size: size as u64,
                    usage,
                    mapped_at_creation: false,
                })
            })
            .collect();
            
        Self {
            buffers,
            current_idx: 0,
            strategy,
            label: label.to_string(),
        }
    }
    
    pub fn current(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_idx]
    }
    
    pub fn next(&mut self) -> &wgpu::Buffer {
        self.current_idx = (self.current_idx + 1) % self.buffers.len();
        self.current()
    }
    
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(self.current(), 0, data);
    }

    pub fn get_current(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_idx]
    }
}
