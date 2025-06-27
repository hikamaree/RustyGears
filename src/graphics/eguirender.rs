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
use egui::Visuals;
use egui::Context;

use egui_wgpu::ScreenDescriptor;
use egui_wgpu::Renderer;
use egui_wgpu::wgpu::TextureView;
use egui_wgpu::wgpu::TextureFormat;
use egui_wgpu::wgpu::Queue;
use egui_wgpu::wgpu::Device;
use egui_wgpu::wgpu::CommandEncoder;
use egui_wgpu::wgpu;

use egui_winit::State;
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;

/// Integrates the `egui` immediate-mode GUI with WGPU rendering.
///
/// `EguiRenderer` handles the full lifecycle of `egui`:
/// - managing context and input state,
/// - processing UI draw commands,
/// - and issuing rendering commands to WGPU.
///
/// It is intended to be used as a subrenderer in a larger render pipeline.
pub struct EguiRenderer {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
}

impl EguiRenderer {
    /// Creates a new [`EguiRenderer`] instance with the given WGPU and window parameters.
    ///
    /// # Arguments
    /// * `device` - The WGPU device used to allocate resources.
    /// * `output_color_format` - The format of the render target.
    /// * `output_depth_format` - Optional depth format (unused by egui).
    /// * `msaa_samples` - MSAA sample count for rendering.
    /// * `window` - The platform window used to drive input handling.
    ///
    /// # Returns
    /// A fully initialized `EguiRenderer`.
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

    /// Processes a single input event from the windowing system.
    ///
    /// This should be called for every event in the main event loop before rendering.
    ///
    /// # Arguments
    /// * `window` - The window that received the event.
    /// * `event` - The window event to pass into egui.
    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    /// Renders the egui UI and submits draw commands to the current frame.
    ///
    /// This method collects input events from the window and feeds them into the egui context.
    /// It then executes user-defined GUI logic provided via `run_ui`, which builds up the interface.
    /// After the UI is constructed, any textures that have changed are uploaded to the GPU.
    /// The shapes produced by egui are tessellated into vertex and index buffers.
    /// A render pass is created targeting the given surface view, and the UI is drawn.
    /// Finally, any textures that have been marked for release by egui are freed from the renderer.
    ///
    /// # Arguments
    /// * `device` - The WGPU device.
    /// * `queue` - The WGPU queue for buffer uploads.
    /// * `encoder` - The command encoder for the current frame.
    /// * `window` - The platform window.
    /// * `window_surface_view` - The target texture view to render into.
    /// * `screen_descriptor` - Screen dimensions and scale factor.
    /// * `run_ui` - A list of GUI components implementing [`Gui`] to render.
    /// * `game` - A read-only view into the ECS and game state.
    pub fn draw<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &'a mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: &mut Vec<Box<dyn Gui + Send + Sync>>,
        game: &GameView,
    ) {
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |_ui| {
            self.context.set_cursor_icon(egui::CursorIcon::None);
            run_ui.into_iter().for_each(|gui_component| {
                gui_component.render_gui(game, &self.context);
            });
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
