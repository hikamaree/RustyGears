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

use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::WindowAttributes;
use winit::window::WindowId;

use crate::render::gpu_resources::GpuResources;
use crate::render::window_state::WindowState;
use crate::EguiRenderer;
use crate::Game;
use crate::GameWindow;
use crate::GearEvent;
use crate::Input;
use crate::Time;

impl ApplicationHandler for Game {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window_id_check = {
            let Ok(game_window) = self.components.get::<GameWindow>() else {
                return;
            };

            if window_id != game_window.id() {
                return;
            }

            game_window.window().clone()
        };

        let Ok(mut gui) = self.components.get_mut::<EguiRenderer>() else {
            return;
        };

        gui.handle_input(&window_id_check, &event);

        drop(gui);

        match event {
            WindowEvent::CloseRequested => {
                self.dispatch_event(GearEvent::Exit);
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Ok(mut time) = self.components.get_mut::<Time>() {
                    time.update();
                };

                self.dispatch_event(GearEvent::Update);

                window_id_check.request_redraw();

                if let Ok(mut input) = self.components.get_mut::<Input>() {
                    input.reset();
                };
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                let Ok(gui) = self.components.get_mut::<EguiRenderer>() else {
                    return;
                };

                if gui.context.wants_keyboard_input() {
                    return;
                }

                let Ok(mut input) = self.components.get_mut::<Input>() else {
                    return;
                };

                match state {
                    winit::event::ElementState::Pressed => input.press_key(key),
                    winit::event::ElementState::Released => input.release_key(key),
                }
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                if let Ok(mut input) = self.components.get_mut::<Input>() {
                    input.update_mouse_delta(delta.0, delta.1);
                };
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Ok(game_window) = self.components.get::<GameWindow>() {
            game_window.request_redraw();
        }
    }

    fn resumed(&mut self, game_loop: &ActiveEventLoop) {
        let title = env!("CARGO_PKG_NAME");

        let window_attributes = WindowAttributes::default().with_title(title);

        let window = game_loop
            .create_window(window_attributes)
            .expect("failed to create window");

        if let Err(e) = window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
            crate::log!(crate::LogKind::Error, "{}", e);
        }

        window.set_cursor_visible(false);

        let game_window = GameWindow::new(window);
        let size = game_window.inner_size();
        let window_for_surface = game_window.window().clone();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window_for_surface)
            .expect("failed to create surface");

        let (adapter, device, queue) = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or_else(|| "Failed to find a suitable GPU adapter.".to_string())?;

                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::default(),
                            memory_hints: Default::default(),
                        },
                        None,
                    )
                    .await
                    .map_err(|e| format!("Failed to create device: {e}"))?;

                Ok::<_, String>((adapter, device, queue))
            })
        }) {
            Ok(v) => v,
            Err(e) => {
                crate::log!(crate::LogKind::Error, "{}", e);
                return;
            }
        };

        let surface_caps = surface.get_capabilities(&adapter);
        let Some(surface_format) = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .or_else(|| surface_caps.formats.get(0).copied())
        else {
            crate::log!(crate::LogKind::Error, "No surface formats available.");
            return;
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let gpu_resources = GpuResources::new(instance, adapter, device, queue);
        let window_state = WindowState::new(surface, config, size);

        self.components.insert(game_window);
        self.components.insert(gpu_resources);
        self.components.insert(window_state);

        while let Some(setup_fn) = self.setupfns.pop_front() {
            setup_fn(self);
        }
    }
}
