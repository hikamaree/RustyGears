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

use egui::TextEdit;
use egui::ScrollArea;
use egui::RichText;
use egui::Context;
use egui::Color32;

use std::collections::HashMap;

use crate::GameView;
use crate::Gui;
use crate::Logs;
use crate::LogKind;

/// A function signature used to define terminal commands.
///
/// Terminal commands take a slice of string slices representing arguments and are required
/// to be thread-safe and static. These functions are typically registered in a command map
/// and invoked through the in-game terminal.
pub type TerminalCommandFn = dyn Fn(&[&str]) + Send + Sync + 'static;

/// A GUI-based terminal for user input and command execution.
///
/// The terminal displays log output and allows users to input commands in a text interface
/// powered by `egui`. It maintains an internal history of input commands that can be navigated
/// using the up and down arrow keys. Commands are matched against a map of registered handlers,
/// which are functions that implement specific in-game functionality.
///
/// Pressing the backtick (`) key toggles the terminal display, and pressing Escape hides it.
/// The terminal automatically focuses the input field when visible, and pressing Enter executes
/// the current command. Log output is colored by kind, including informational messages, warnings,
/// errors, and command I/O.
pub struct TerminalGui {
    input: String,
    display: bool,
    command_map: HashMap<String, Box<TerminalCommandFn>>,
    history_index: Option<usize>,
}

impl TerminalGui {
    /// Constructs a new terminal with a map of available commands.
    ///
    /// The command map associates command strings with functions that handle their behavior.
    /// The map is owned by the terminal and used to look up handlers during execution.
    pub fn new(command_map: HashMap<String, Box<TerminalCommandFn>>) -> Self {
        Self {
            input: String::new(),
            display: false,
            command_map,
            history_index: None,
        }
    }

    /// Returns the color associated with a given log kind.
    ///
    /// Each `LogKind` variant is mapped to a distinct color for easier visual identification
    /// in the terminal UI.
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

    /// Parses and executes a single line of input as a terminal command.
    ///
    /// The line is split into a command name and argument list. If the command is found
    /// in the registered map, the corresponding function is invoked with the arguments.
    /// If no matching command exists, an error is logged.
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

        let Ok(logs) = game.get::<Logs>() else {
            return;
        };

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            let idx = match self.history_index {
                Some(idx) => idx,
                None => 0,
            };

            let mut count = 0;
            let mut target = None;

            for (kind, line) in logs.all().iter().rev() {
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

                    for (kind, line) in logs.all().iter().rev() {
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
                                for (kind, line) in logs.all() {
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
