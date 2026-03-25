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

use crate::Texture;

pub struct SetResolution {
    pub resolution: winit::dpi::PhysicalSize<u32>,
}

impl crate::Command for SetResolution {
    fn apply(self: Box<Self>, game: &mut crate::Game) {
        let Ok(mut window) = game
            .components
            .get_mut::<crate::render::window_state::WindowState>()
        else {
            return;
        };
        let Ok(gpu) = game
            .components
            .get::<crate::render::gpu_resources::GpuResources>()
        else {
            return;
        };
        let Ok(mut state) = game
            .components
            .get_mut::<crate::render::render_state::RenderState>()
        else {
            return;
        };

        if self.resolution.width > 0 && self.resolution.height > 0 {
            window.size = self.resolution;
            window.config.width = self.resolution.width;
            window.config.height = self.resolution.height;
            window.surface.configure(&gpu.device, &window.config);
            state.depth_texture =
                Texture::create_depth_texture(&gpu.device, &window.config, "depth_texture");
            state
                .projection
                .resize(self.resolution.width, self.resolution.height);
        }
    }
}

#[macro_export]
macro_rules! set_resolution {
    ( $x:expr, $y:expr ) => {{
        $crate::send_command($crate::render::set_resolution::SetResolution {
            resolution: winit::dpi::PhysicalSize::new($x, $y),
        });
    }};
}
