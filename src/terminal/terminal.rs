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

use crate::Logs;
use crate::LogKind;
use crate::Game;
use std::collections::HashMap;
use cgmath::vec3;
use cgmath::One;
use cgmath::Quaternion;
use egui::{Color32, Context, RichText, ScrollArea, TextEdit};
use std::collections::VecDeque;
use crate::Command;
use crate::CommandFunction;
use crate::GameView;
use crate::Gui;

pub type TerminalCommandFn = dyn Fn(&[&str], &mut VecDeque<Box<dyn Command + 'static>>) + Send + Sync + 'static;

pub struct TerminalGui {
    input: String,
    display: bool,
    command_map: HashMap<String, Box<TerminalCommandFn>>,
}

impl TerminalGui {
    pub fn new(command_map: HashMap<String, Box<TerminalCommandFn>>) -> Self {
        Self {
            input: String::new(),
            display: false,
            command_map,
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
            Box::new(|args: &[&str], _cmds: &mut VecDeque<Box<dyn Command + 'static>>| {
                crate::log!(LogKind::Output, "{}", args.join(" "));
            }),
        );

        map.insert(
            "spawn_miku".to_string(),
            Box::new(|_args: &[&str], cmds: &mut VecDeque<Box<dyn Command + 'static>>| {
                cmds.push_back(Box::new(CommandFunction {
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
                }));
            }),
            );

        map
    }

    fn execute_command(&mut self, command: &str, commands: &mut VecDeque<Box<dyn Command + 'static>>) {
        crate::log!(LogKind::Input, "> {}", command);

        let tokens: Vec<&str> = command.trim().split_whitespace().collect();
        if tokens.is_empty() {
            return;
        }

        let cmd = tokens[0];
        let args = &tokens[1..];

        match self.command_map.get(cmd) {
            Some(handler) => {
                handler(args, commands);
            }
            None => {
                crate::log!(LogKind::Error, "Unknown command: {}", cmd);
            }
        }
    }
}

impl Gui for TerminalGui {
    fn render_gui(&mut self, game: &GameView, ctx: &Context, commands: &mut VecDeque<Box<dyn Command + 'static>>) {
        if !self.display && ctx.input(|i| i.key_pressed(egui::Key::Backtick)) {
            self.display = true;
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
            });
    }
}
