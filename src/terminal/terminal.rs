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

use cgmath::vec3;
use cgmath::One;
use cgmath::Quaternion;

use egui::TextEdit;
use egui::ScrollArea;
use egui::RichText;
use egui::Context;
use egui::Color32;

use std::any::Any;
use std::collections::HashMap;

use crate::send_command;
use crate::CommandFunction;
use crate::GameView;
use crate::Gui;
use crate::Logs;
use crate::LogKind;
use crate::Game;

pub type TerminalCommandFn = dyn Fn(&[&str]) + Send + Sync + 'static;

pub struct TerminalGui {
    input: String,
    display: bool,
    command_map: HashMap<String, Box<TerminalCommandFn>>,
    history_index: Option<usize>,
}

impl TerminalGui {
    pub fn new(command_map: HashMap<String, Box<TerminalCommandFn>>) -> Self {
        Self {
            input: String::new(),
            display: false,
            command_map,
            history_index: None,
        }
    }

    fn color_for_kind(kind: &LogKind) -> Color32 {
        match kind {
            LogKind::Info => Color32::CYAN,
            LogKind::Warning => Color32::YELLOW,
            LogKind::Error => Color32::RED,
            LogKind::Debug => Color32::BLUE,
            LogKind::Input => Color32::GREEN,
            LogKind::Output => Color32::WHITE,
        }
    }

    pub fn default_commands() -> HashMap<String, Box<TerminalCommandFn>> {
        let mut map: HashMap<String, Box<TerminalCommandFn>> = HashMap::new();

        map.insert(
            "echo".to_string(),
            Box::new(|args: &[&str]| {
                crate::log!(LogKind::Output, "{}", args.join(" "));
            }),
        );

        map.insert(
            "spawn_miku".to_string(),
            Box::new(|_args: &[&str]| {
                send_command(CommandFunction {
                    run: Box::new(move |game: &mut Game| {
                        let Ok(scene) = game.components.get_mut::<crate::WorldScene>() else {
                            return;
                        };

                        let transform = crate::Transform {
                            position: vec3(15.0, 15.0, 15.0),
                            rotation: Quaternion::one(),
                            scale: vec3(300.0, 300.0, 300.0),
                        };

                        scene.spawn()
                            .with(crate::Model3d { path: "miku/miku".into() })
                            .with(transform);
                    }),
                });
            }),
        );

        map.insert(
            "show_stats".to_string(),
            Box::new(|args: &[&str]| {
                let Some(arg0) = args.get(0) else {
                    crate::log!(LogKind::Warning, "Usage: show_stats [true|false|1|0]");
                    return;
                };

                let arg = arg0.to_lowercase();
                let show = match arg.as_str() {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => {
                        crate::log!(LogKind::Warning, "Usage: show_stats [true|false|1|0]");
                        return;
                    }
                };

                send_command( CommandFunction {
                    run: Box::new(move |game: &mut Game| {
                        let Ok(scene) = game.components.get_mut::<crate::WorldScene>() else {
                            return;
                        };

                        for gui in &mut scene.render_gui {
                            if let Some(engine_stats) = (&mut **gui as &mut dyn Any).downcast_mut::<crate::EngineStats>() {
                                engine_stats.show(show);
                            }
                        }
                    }),
                });
            }),
        );

        map
    }

    fn execute_command(&mut self, command: &str) {
        crate::log!(LogKind::Input, "{}", command);

        let tokens: Vec<&str> = command.trim().split_whitespace().collect();
        if tokens.is_empty() {
            return;
        }

        let cmd = tokens[0];
        let args = &tokens[1..];

        match self.command_map.get(cmd) {
            Some(handler) => {
                handler(args);
            }
            None => {
                crate::log!(LogKind::Error, "Unknown command: {}", cmd);
            }
        }
    }
}

impl Gui for TerminalGui {
    fn render_gui(&mut self, game: &GameView, ctx: &Context) {
        if !self.display && ctx.input(|i| i.key_pressed(egui::Key::Backtick)) {
            self.display = true;
            self.history_index = Some(0);
        }
        if self.display && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.display = false;
        }

        if !self.display {
            return;
        }

        let logs = match game.get::<Logs>() {
            Some(logs) => logs.all(),
            None => return,
        };

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            let idx = match self.history_index {
                Some(idx) => idx,
                None => 0,
            };

            let mut count = 0;
            let mut target = None;

            for (kind, line) in logs.iter().rev() {
                if matches!(kind, LogKind::Input) {
                    if count == idx {
                        target = Some(line);
                        break;
                    }
                    count += 1;
                }
            }

            if let Some(line) = target {
                self.input = line.to_string();
                self.history_index = Some(idx + 1);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if let Some(current) = self.history_index {
                if current > 0 {
                    let new_index = current - 1;

                    let mut count = 0;
                    let mut target = None;

                    for (kind, line) in logs.iter().rev() {
                        if matches!(kind, LogKind::Input) {
                            if count == new_index {
                                target = Some(line);
                                break;
                            }
                            count += 1;
                        }
                    }

                    if let Some(line) = target {
                        self.input = line.to_string();
                        self.history_index = Some(new_index);
                    } else {
                        self.input.clear();
                        self.history_index = None;
                    }
                } else {
                    self.input.clear();
                    self.history_index = None;
                }
            }
        }

        egui::Area::new("Terminal".into())
            .fixed_pos([10.0, 10.0])
            .show(ctx, |ui| {
                egui::Frame::window(&ctx.style())
                    .fill(Color32::from_black_alpha(250))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .max_height(250.0)
                            .show(ui, |ui| {
                                for (kind, line) in logs {
                                    ui.label(
                                        RichText::new(line)
                                        .monospace()
                                        .color(Self::color_for_kind(kind)),
                                    );
                                }
                            });

                        ui.separator();

                        let input_field = ui.add(
                            TextEdit::singleline(&mut self.input)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Enter command...")
                            .desired_width(f32::INFINITY)
                        );

                        if input_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let command = self.input.trim().to_string();
                            if !command.is_empty() {
                                self.execute_command(&command);
                                self.input.clear();
                                self.history_index = Some(0);
                            }
                        } else if !input_field.has_focus() {
                            input_field.request_focus();
                        }
                    });
            });
    }
}
