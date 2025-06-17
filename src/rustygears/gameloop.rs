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

use winit::window::WindowAttributes;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::window::WindowId;
use winit::event_loop::ActiveEventLoop;
use winit::application::ApplicationHandler;
use winit::keyboard::PhysicalKey;
use winit::event::KeyEvent;

use crate::GearEvent;
use crate::Game;
use crate::Graphics;
use crate::EguiRenderer;
use crate::Input;

impl ApplicationHandler for Game {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Ok(graphics) = self.components.get_mut::<Graphics>() else {
            return;
        };

        let Ok(input) = self.components.get_mut::<Input>() else {
            return;
        };

        if window_id != graphics.window.id() {
            return;
        }

        let Some(gui) = self.gui.as_mut() else {
            return;
        };
        gui.handle_input(&graphics.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                Game::dispatch_event(self, GearEvent::Exit());
                event_loop.exit()
            }

            WindowEvent::Resized(physical_size) => {
                graphics.resize(physical_size);
            }

            WindowEvent::RedrawRequested => {
                self.update();
            }

            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } => {
                if gui.context.wants_keyboard_input() {
                    return;
                }
                match state {
                    winit::event::ElementState::Pressed => input.press_key(key),
                    winit::event::ElementState::Released => input.release_key(&key),
                }
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        let Ok(input) = self.components.get_mut::<Input>() else {
            return;
        };

        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                input.update_mouse_delta(delta.0, delta.1);
            }

            _ => {}
        }
    }

    fn resumed(&mut self, game_loop: &ActiveEventLoop) {
        let title = env!("CARGO_PKG_NAME");

        let window_attributes = WindowAttributes::default()
            .with_title(title);

        let window = game_loop
            .create_window(window_attributes)
            .expect("failed to create window");

        if let Err(e) = window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
            eprintln!("{}", e);
        }

        window.set_cursor_visible(false);

        let Ok(rt) = tokio::runtime::Runtime::new() else {
            return;
        };

        let Ok(graphics) = rt.block_on(Graphics::new(window.into())) else {
            return;
        };

        let egui = EguiRenderer::new(&graphics.device, graphics.config.format, None, 1, &graphics.window);

        self.components.insert(graphics);

        self.gui = Some(egui);

        while let Some(setup_fn) = self.setupfns.pop_front() {
            setup_fn(self);
        }
    }
}
