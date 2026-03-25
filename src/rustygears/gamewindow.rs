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

use std::sync::Arc;
use winit::window::Window;

#[derive(Clone)]
pub struct GameWindow {
    window: Arc<Window>,
}

impl GameWindow {
    pub fn new(window: Window) -> Self {
        Self {
            window: Arc::new(window),
        }
    }

    pub fn from_arc(window: Arc<Window>) -> Self {
        Self { window }
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub fn inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    pub fn set_cursor_grab(
        &self,
        mode: winit::window::CursorGrabMode,
    ) -> Result<(), winit::error::ExternalError> {
        self.window.set_cursor_grab(mode)
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
