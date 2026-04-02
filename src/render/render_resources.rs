// SPDX-License-Identifier: GPL-3.0-or-later
//
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

use super::layout::LayoutRegistry;
use crate::Buffer;
use crate::BufferStrategy;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct RenderResources {
    pub buffers: RwLock<HashMap<String, Buffer>>,
    pub layouts: LayoutRegistry,
}

impl RenderResources {
    pub fn new(layouts: LayoutRegistry) -> Self {
        Self {
            buffers: RwLock::new(HashMap::new()),
            layouts,
        }
    }

    pub fn create_buffer(
        &self,
        device: &wgpu::Device,
        name: &str,
        size: usize,
        usage: wgpu::BufferUsages,
        strategy: BufferStrategy,
    ) {
        let buffer = Buffer::new(device, size, usage, strategy, name);
        if let Ok(mut buffers) = self.buffers.write() {
            buffers.insert(name.to_string(), buffer);
        } else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire buffers write lock"
            );
        }
    }

    pub fn get_buffer(&self, name: &str) -> Option<Buffer> {
        self.buffers.read().ok()?.get(name).cloned()
    }

    pub fn get_or_create_buffer<F>(&self, key: &str, create_fn: F) -> Buffer
    where
        F: FnOnce() -> Buffer,
    {
        let Ok(mut buffers) = self.buffers.write() else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire buffers write lock"
            );
            return create_fn();
        };
        buffers
            .entry(key.to_string())
            .or_insert_with(create_fn)
            .clone()
    }

    pub fn get_or_create_buffer_with<F>(
        &self,
        _queue: &wgpu::Queue,
        key: &str,
        create_fn: F,
        update_fn: impl FnOnce(&mut Buffer),
    ) -> Buffer
    where
        F: FnOnce() -> Buffer,
    {
        let Ok(mut buffers) = self.buffers.write() else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire buffers write lock"
            );
            return create_fn();
        };
        let buffer = buffers.entry(key.to_string()).or_insert_with(create_fn);
        update_fn(buffer);
        buffer.clone()
    }

    pub fn update_buffer<F>(&self, key: &str, update_fn: F)
    where
        F: FnOnce(&mut Buffer),
    {
        let Ok(mut buffers) = self.buffers.write() else {
            crate::log!(
                crate::LogKind::Error,
                "Failed to acquire buffers write lock"
            );
            return;
        };
        if let Some(buffer) = buffers.get_mut(key) {
            update_fn(buffer);
        }
    }

    pub fn create_bind_group<L: crate::render::layout::HasBindGroupLayout>(
        &self,
        device: &wgpu::Device,
        name: &str,
    ) -> Option<wgpu::BindGroup>
    where
        L: 'static,
    {
        let Ok(buffers) = self.buffers.read() else {
            crate::log!(crate::LogKind::Error, "Failed to acquire buffers read lock");
            return None;
        };
        let buffer = buffers.get(name)?;
        let resource = buffer.current().as_entire_binding();

        let layout = self.layouts.get_opt::<L>()?;

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", name)),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource,
            }],
        }))
    }
}
