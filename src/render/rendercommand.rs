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

use egui_wgpu::ScreenDescriptor;
use crate::EguiRenderer;
use crate::DrawModel;
use crate::BufferStrategy;
use crate::Buffer;
use crate::Graphics;
use crate::InstanceRaw;
use crate::Game;
use crate::GameView;
use crate::Command;
use crate::WorldScene;
use crate::RenderBatch;

/// A render command containing all data necessary to draw the entire frame.
///
/// This is the top-level command issued to the renderer each frame,
/// typically produced by the `Render` gear. It consists of one or more `RenderBatch`es,
/// each of which groups models by compatible pipeline and camera settings to improve draw efficiency.
///
/// # Note
/// This command should **only** be constructed and sent by the default `Render` gear.
/// If you're implementing a custom rendering pipeline or replacing the default renderer,
/// you may emit your own `RenderCommand`, but in most cases users **should not** send
/// this command manually.
///
/// # Fields
/// - `batches`: A list of render batches grouped by pipeline and camera. Each batch
///   contains preprocessed model/instance data ready to be drawn.
#[derive(Debug)]
pub struct RenderCommand {
    /// A list of render batches, each representing a group of models that share the same
    /// render pipeline and camera.
    ///
    /// Batches help reduce GPU state changes by grouping compatible draw calls together.
    pub batches: Vec<RenderBatch>,
}


impl Command for RenderCommand {
    fn apply(self: Box<Self>, game: &mut Game) {
        let Ok(scene) = game.components.get_mut::<WorldScene>() else {
            return;
        };

        let Ok(graphics) = game.components.get_mut::<Graphics>() else {
            return;
        };

        graphics.t_count = 0;

        let output = match graphics.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                crate::log!(crate::LogKind::Error, "Failed to get current texture: {e:?}");
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if let (Some(camera_bg), Some(light_bg)) = (
            graphics.bind_groups.get(&crate::BindGroupLayoutKey::Camera),
            graphics.bind_groups.get(&crate::BindGroupLayoutKey::Light)
        ) {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &graphics.depth_texture.view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for batch in &self.batches {
                if let Some(pipeline) = graphics.pipelines.get(&batch.tag) {
                    render_pass.set_pipeline(pipeline);

                    for model_data in &batch.prepared_models {
                        let key = format!("{:?}:lod{}:mesh{}", model_data.model3d, model_data.lod_index, model_data.mesh_ranges[0].mesh_index);
                        let buffer = graphics.buffers.entry(key)
                            .and_modify(|b| {
                                b.ensure_capacity(&graphics.device, model_data.instance_data.len() * std::mem::size_of::<InstanceRaw>());
                                b.next();
                                b.write(&graphics.queue, bytemuck::cast_slice(&model_data.instance_data));
                            }).or_insert_with(|| {
                                Buffer::new(
                                    &graphics.device,
                                    model_data.instance_data.len().next_power_of_two() * std::mem::size_of::<InstanceRaw>(),
                                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                                    BufferStrategy::Triple,
                                    "majmun",
                                )
                            });

                        render_pass.set_vertex_buffer(1, buffer.current().slice(..));

                        if let Some(render_object) = scene.get_render_object(&model_data.model3d) {
                            graphics.t_count += render_pass.draw_model_instanced(
                                &render_object.lods[model_data.lod_index],
                                camera_bg,
                                light_bg,
                                &model_data.mesh_ranges,
                            );
                        }
                    }
                }
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [graphics.config.width, graphics.config.height],
            pixels_per_point: graphics.window.scale_factor() as f32,
        };

        let gameview = GameView {
            components: game.components.get_view()
        };

        if let Ok(gui) = game.components.get_mut::<EguiRenderer>() {
            gui.draw(
                &graphics.device,
                &graphics.queue,
                &mut encoder,
                &graphics.window,
                &view, screen_descriptor,
                &mut scene.render_gui,
                &gameview,
            );
        };
        
        graphics.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
