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

use super::registry::PassRegistry;
use super::targets::RenderTargetPool;
use crate::Projection;
use crate::RenderConfig;
use crate::Texture;
use std::sync::Arc;
use std::sync::RwLock;

pub struct RenderState {
    pub registry: Arc<RwLock<PassRegistry>>,
    pub frame_count: u64,
    pub triangles_rendered: u32,
    pub config: RenderConfig,
    pub depth_texture: Texture,
    pub projection: Projection,
    pub target_pool: RenderTargetPool,
}

impl RenderState {
    pub fn new(depth_texture: Texture, projection: Projection, config: RenderConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PassRegistry::new())),
            frame_count: 0,
            triangles_rendered: 0,
            config,
            depth_texture,
            projection,
            target_pool: RenderTargetPool::new(),
        }
    }
}
