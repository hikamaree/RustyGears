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

use crate::GameView;
use crate::Gui;
use egui::epaint::Shadow;
use egui::{Context, Visuals};
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::Renderer;

use egui_winit::State;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use egui_wgpu::wgpu;
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;

pub struct EguiRenderer {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        let visuals = Visuals {
            window_shadow: Shadow::NONE,
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = State::new(egui_context.clone(), id, &window, None, None, None);

        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        EguiRenderer {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &'a mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: &Vec<Box<dyn Gui + Send + Sync>>,
        game: &GameView,
    ) {
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |_ui| {
            self.context.set_cursor_icon(egui::CursorIcon::None);
            for gui_component in run_ui {
                gui_component.render_gui(game, &self.context);
            }
        });

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }

        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let mut rpass = rpass.forget_lifetime();
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
