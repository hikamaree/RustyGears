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

use egui::Color32;
use egui::RichText;
use egui::Rect;
use egui::Vec2;
use egui::Rgba;
use egui::Area;

use crate::Graphics;
use crate::Gui;
use crate::GameView;
use crate::Time;

use super::gpu::GpuInfo;

use std::sync::Mutex;

pub struct EngineStats {
    gpu: Mutex<GpuInfo>,
}

impl EngineStats {
    pub fn new() -> Self {
        Self {
            gpu: Mutex::new(GpuInfo::new()),
        }
    }
}

impl Gui for EngineStats {
    fn render_gui(&self, game: &GameView, ctx: &egui::Context) {
        let Ok(mut gpu) = self.gpu.lock() else {
            return;
        };

        let Some(time) = game.get::<Time>() else {
            return;
        };

        let Some(graphics) = game.get::<Graphics>() else {
            return;
        };

        gpu.update(time.total_time());

        Area::new("game_stats".into())
            .fixed_pos([0.0, 0.0])
            .show(ctx, |ui| {
                let padding = 10.0;
                let text_size = Vec2::new(200.0, 100.0);

                let rect = Rect::from_min_size(
                    egui::pos2(10.0, 10.0),
                    text_size + Vec2::splat(padding * 2.0)
                );

                let bg_color = Rgba::from_rgba_premultiplied(0.0, 0.0, 0.0, 0.5);
                ui.painter().rect_filled(
                    rect,
                    10.0,
                    bg_color
                );

                ui.put(rect.shrink(padding), |ui: &mut egui::Ui| {
                    ui.vertical(|ui: &mut egui::Ui| -> egui::Response {
                        ui.label(RichText::new(format!("FPS: {}", time.fps()))
                            .monospace()
                            .color(Color32::WHITE));
                        ui.label(RichText::new(format!("{}", gpu.display()))
                            .monospace()
                            .color(Color32::WHITE));
                        ui.label(RichText::new(format!("Rendering:\n  Triangles: {}", graphics.t_count))
                            .monospace()
                            .color(Color32::WHITE))
                    }).response
                });
            });
    }
}
