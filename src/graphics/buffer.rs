// SPDX-License-Identifier: GPL-3.0-or-later

// This file is part of Rusty Gears.
//
// Rusty Gears is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rusty Gears is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub const DEFAULT_INSTANCE_BUFFER_SIZE: usize = 32 * 1024 * 1024;
pub const DEFAULT_CAMERA_BUFFER_SIZE: usize = 80;

/// Defines the buffer usage strategy:
/// - `Single`: Single buffer (no rotation).
/// - `Double`: Double buffering (helps avoid GPU stalls).
/// - `Triple`: Triple buffering (ideal for frame pipelining).
#[derive(Debug, Clone)]
pub enum BufferStrategy {
    Single,
    Double,
    Triple,
}

/// A GPU buffer abstraction that supports single, double, or triple buffering.
/// Useful for dynamic data (e.g., instance transforms) to avoid GPU/CPU sync issues.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub buffers: Vec<wgpu::Buffer>,
    pub current_idx: usize,
    pub strategy: BufferStrategy,
    pub label: String,
    pub size: usize,
}

impl Buffer {

    /// Creates a new buffer (or multiple buffers) according to the selected strategy.
    ///
    /// # Arguments
    /// * `device` - The wgpu device used to allocate the buffers.
    /// * `size` - Size of each buffer in bytes.
    /// * `usage` - Buffer usage flags (e.g., `VERTEX`, `COPY_DST`, etc.).
    /// * `strategy` - Buffer rotation strategy.
    /// * `label` - A debug label prefix for the buffers.
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
            size,
        }
    }

    /// Returns the current buffer in rotation.
    pub fn current(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_idx]
    }
    
    /// Advances to the next buffer (for double/triple buffering) and returns it.
    pub fn next(&mut self) -> &wgpu::Buffer {
        self.current_idx = (self.current_idx + 1) % self.buffers.len();
        self.current()
    }
    
    /// Writes raw byte data into the current buffer.
    ///
    /// # Arguments
    /// * `queue` - GPU queue used to write to the buffer.
    /// * `data` - Raw byte slice to upload.
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(self.current(), 0, data);
    }

    /// Ensures the buffer has at least `needed` bytes of capacity.
    /// If not, resizes all buffers in the pool to the next power of two.
    ///
    /// This function is safe to call every frame — it only reallocates if needed.
    ///
    /// # Arguments
    /// * `device` - GPU device used for reallocation.
    /// * `needed` - Required byte size.
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed > self.size {
            self.size = needed.next_power_of_two().max(256);

            let count = match self.strategy {
                BufferStrategy::Single => 1,
                BufferStrategy::Double => 2,
                BufferStrategy::Triple => 3,
            };

            self.buffers = (0..count)
                .map(|i| {
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&format!("{}_buffer_{}", self.label, i)),
                        size: self.size as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })
                })
                .collect();

            self.current_idx = 0;
        }
    }
}
