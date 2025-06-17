use cgmath::vec3;
use cgmath::One;
use cgmath::Quaternion;
use egui::{Color32, Context, RichText, ScrollArea, TextEdit};
use std::collections::VecDeque;
use crate::Command;
use crate::CommandFunction;
use crate::GameView;
use crate::Gui;
use crate::Model3d;

pub struct TerminalGui {
    input: String,
    output: VecDeque<String>,
    max_lines: usize,
    scroll_to_bottom: bool,
    display: bool,
}

pub struct TestCmd;
impl Command for TestCmd {
    fn apply(self: Box<Self>, _game: &mut crate::Game) {
        println!("majmuneeee");
    }
}

impl TerminalGui {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            output: VecDeque::with_capacity(100),
            max_lines: 100,
            scroll_to_bottom: false,
            display: false,
        }
    }

    fn execute_command(&mut self, command: &str, commands: &mut VecDeque<Box<dyn Command + 'static>>) {
        self.log(format!("> {}", command));
        self.log(format!("Executed: {}", command));

        match command {
            "test" => commands.push_back(Box::new(TestCmd)),
            "spawn miku" => commands.push_back(Box::new( CommandFunction { 
                run: Box::new(move |game| {
                    let Ok(scene) = game.components.get_mut::<crate::WorldScene>() else {
                        return;
                    };

                    let transform = crate::Transform { 
                        position: vec3(15.0, 15.0, 15.0),
                        rotation: Quaternion::one(),
                        scale: vec3(300.0, 300.0, 300.0),
                    };

                    scene.spawn()
                        .with(Model3d {path: "miku/miku".into()})
                        .with(transform);
                })
            }
            )),
            _ => {}
        };
    }

    fn log(&mut self, line: impl Into<String>) {
        if self.output.len() >= self.max_lines {
            self.output.pop_front();
        }
        self.output.push_back(line.into());
        self.scroll_to_bottom = true;
    }
}

impl Gui for TerminalGui {
    fn render_gui(&mut self, _game: &GameView, ctx: &Context, commands: &mut VecDeque<Box<dyn Command + 'static>>) {
        if !self.display && ctx.input(|i| i.key_pressed(egui::Key::Backtick)) {
            self.display = true;
        }
        if self.display && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.display = false;
        }

        if !self.display {
            return;
        }

        egui::Window::new("Terminal")
            .default_pos([20.0, 20.0])
            .default_size([600.0, 200.0])
            .resizable(true)
            .collapsible(false)
            .min_width(400.0)
            .min_height(150.0)
            .max_width(800.0)
            .max_height(400.0)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for line in &self.output {
                            ui.label(
                                RichText::new(line)
                                .monospace()
                                .color(Color32::LIGHT_GREEN),
                            );
                        }
                    });

                ui.separator();

                let input_field = ui.add(
                    TextEdit::singleline(&mut self.input)
                    .hint_text("Enter command...")
                    .desired_width(f32::INFINITY),
                );


                if input_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let command = self.input.trim().to_string();
                    if !command.is_empty() {
                        self.execute_command(&command, commands);
                        self.input.clear();
                    }
                } else if !input_field.has_focus() {
                    input_field.request_focus();
                }
            });
    }
}
