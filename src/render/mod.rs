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

mod render;
mod rendercommand;
mod config;

pub mod graph;
pub mod passes;

pub mod filter;
pub mod framedata;
pub mod layout;
pub mod lod;
pub mod pass;
pub mod pass_id;
pub mod registry;
pub mod sort;
pub mod targets;
pub mod cull;

pub mod set_resolution;
mod buffer;
mod eguirender;
mod gui;
mod vertex;
mod renderbatch;
mod light;
mod draw_wgpu;
pub mod gpu_resources;
pub mod window_state;
pub mod render_resources;
pub mod render_state;

pub use config::RenderConfig;
pub use filter::EntityFilter;
pub use framedata::collect_lights;
pub use layout::LayoutRegistry;
pub use pass::PassContext;
pub use pass::PassData;
pub use pass::RenderPass;
pub use pass_id::PassId;
pub use pass::PassOutput;
pub use registry::PassRegistry;
pub use render::Render;
pub use rendercommand::ExecuteRender;
pub use targets::OutputHandle;
pub use targets::RenderTarget;
pub use targets::RenderTargetPool;
pub use targets::TargetDescriptor;
pub use targets::TargetSize;

pub use buffer::*;
pub use eguirender::*;
pub use gui::*;
pub use vertex::*;
pub use renderbatch::*;
pub use light::*;

pub use gpu_resources::GpuResources as GpuDevice;
pub use window_state::WindowState;
pub use render_resources::RenderResources;
pub use render_state::RenderState;
