use crate::GameView;
use egui::Context;

pub trait Gui: Send + Sync {
    fn render_gui(&self, game: &GameView, context: &Context);
}
