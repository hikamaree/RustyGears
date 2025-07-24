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

use cgmath::One;
use cgmath::Quaternion;
use cgmath::vec3;

use std::any::Any;
use std::collections::HashMap;

use crate::Game;
use crate::CommandFunction;
use crate::LogKind;
use crate::TerminalCommandFn;

pub fn default_terminal_commands() -> HashMap<String, Box<TerminalCommandFn>> {
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
            crate::send_command(CommandFunction {
                run: Box::new(move |game: &mut Game| {
                    let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
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

            crate::send_command( crate::CommandFunction {
                run: Box::new(move |game: &mut Game| {
                    let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else {
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

    map.insert(
        "set_resolution".to_string(),
        Box::new(|args: &[&str]| {
            if args.len() != 2 {
                crate::log!(LogKind::Warning, "Usage: set_resolution <width> <height>");
                return;
            }

            let parse_u32 = |s: &str| s.parse::<u32>().map_err(|_| {
                crate::log!(LogKind::Warning, "Invalid resolution value: {}", s)
            });

            let Ok(width) = parse_u32(args[0]) else { return };
            let Ok(height) = parse_u32(args[1]) else { return };

            crate::set_resolution!(width, height);
        }),
    );

    map
}
