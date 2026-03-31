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

use egui::Area;
use egui::Color32;
use egui::RichText;

use crate::render::render_state::RenderState;
use crate::GameView;
use crate::Gui;
use crate::Time;

use super::gpu::GpuInfo;

pub struct EngineStats {
    show: bool,
    gpu: GpuInfo,
}

impl EngineStats {
    pub fn new() -> Self {
        Self {
            show: true,
            gpu: GpuInfo::new(),
        }
    }

    pub fn show(&mut self, show: bool) {
        self.show = show;
    }
}

impl Gui for EngineStats {
    fn render_gui(&mut self, game: &GameView, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        let Ok(time) = game.get::<Time>() else {
            return;
        };

        let Ok(render_state) = game.get::<RenderState>() else {
            return;
        };

        self.gpu.update(time.total_time());

        Area::new("engine_stats".into())
            .fixed_pos([10.0, 10.0])
            .show(ctx, |ui| {
                egui::Frame::window(&ctx.style())
                    .fill(Color32::from_black_alpha(200))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_min_width(200.0);
                            ui.label(
                                RichText::new(format!("FPS: {}", time.fps()))
                                    .monospace()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new(format!("{}", self.gpu.display()))
                                    .monospace()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Rendering:\n  Frame: {}\n  Triangles: {}",
                                    render_state.frame_count, render_state.triangles_rendered
                                ))
                                .monospace()
                                .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new(format!("Frametime: {}", time.frametime()))
                                    .monospace()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                RichText::new("Gear update times:")
                                    .monospace()
                                    .color(Color32::WHITE),
                            );
                            for (gear, micros) in time.gear_update_times() {
                                ui.label(
                                    RichText::new(format!("  {}: {:.0}", gear, micros))
                                        .monospace()
                                        .color(Color32::WHITE),
                                );
                            }
                        });
                    });
            });
    }
}
