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
use crate::Projection;
use crate::Texture;
use std::sync::Arc;
use std::sync::RwLock;

pub struct RenderState {
    pub registry: Arc<RwLock<PassRegistry>>,
    pub t_count: u32,
    pub depth_texture: Texture,
    pub projection: Projection,
}

impl RenderState {
    pub fn new(depth_texture: Texture, projection: Projection) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PassRegistry::new())),
            t_count: 0,
            depth_texture,
            projection,
        }
    }
}
