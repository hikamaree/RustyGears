use egui::Color32;
use egui::RichText;
use egui::Rect;
use egui::Vec2;
use egui::Rgba;
use egui::Area;

use crate::Gui;
use crate::GameView;

use super::gpu::GpuInfo;

pub struct EngineStats {
    gpu: GpuInfo,
}

impl EngineStats {
    pub fn new() -> Self {
        Self {
            gpu: GpuInfo::new(),
        }
    }
}

impl Gui for EngineStats {
    fn render_gui(&self, game: &GameView, ctx: &egui::Context) {
        Area::new("game_stats".into())
            .fixed_pos([0.0, 0.0])
            .show(ctx, |ui| {
                let padding = 10.0;
                let text_size = Vec2::new(175.0, 80.0);

                let rect = Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
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
                        ui.label(RichText::new(format!("FPS: {}", game.time.fps()))
                            .monospace()
                            .color(Color32::WHITE));
                            ui.label(RichText::new(format!("{}", self.gpu.display()))
                                .monospace()
                                .color(Color32::WHITE))
                    }).response
                });
            });
    }
}
