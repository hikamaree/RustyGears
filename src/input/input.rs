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

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Represents the change in mouse position since the last frame or update cycle.
///
/// This struct is typically updated by the input system during mouse motion events
/// and is reset at the end of each frame.
#[derive(Debug, Clone, Copy)]
pub struct MouseDelta {
    pub dx: f64,
    pub dy: f64,
}

/// Tracks the current input state including keyboard keys and mouse movement.
///
/// The `Input` struct is responsible for collecting and exposing per-frame input data.
/// It stores currently pressed keys and accumulated mouse movement deltas.
/// Mouse deltas are reset every frame by the main loop or input system.
pub struct Input {
    mouse_delta: MouseDelta,
    pressed_keys: HashSet<KeyCode>
}

impl Input {
    /// Creates a new `Input` instance with no keys pressed and zero mouse movement.
    pub fn new() -> Self {
        let mouse_delta = MouseDelta {
            dx: 0.0,
            dy: 0.0
        };

        let pressed_keys = HashSet::new();

        Self {
            mouse_delta,
            pressed_keys,
        }
    }

    /// Updates the accumulated mouse delta by adding the provided values.
    ///
    /// Called internally when a mouse motion event is received.
    ///
    /// # Arguments
    /// * `dx` - Change in the horizontal mouse position.
    /// * `dy` - Change in the vertical mouse position.
    pub(crate) fn update_mouse_delta(&mut self, dx: f64, dy: f64) {
        self.mouse_delta.dx += dx;
        self.mouse_delta.dy += dy;
    }

    /// Resets the mouse delta back to zero.
    ///
    /// This is typically called at the end of a frame to prepare for the next update cycle.
    pub(crate) fn reset_mouse_delta(&mut self) {
        self.mouse_delta.dx = 0.0;
        self.mouse_delta.dy = 0.0;
    }

    /// Marks a key as currently pressed.
    ///
    /// Called when a key press event is received.
    ///
    /// # Arguments
    /// * `key` - The key code of the pressed key.
    pub(crate) fn press_key(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);        
    }

    /// Marks a key as released.
    ///
    /// Called when a key release event is received.
    ///
    /// # Arguments
    /// * `key` - A reference to the released key code.
    pub(crate) fn release_key(&mut self, key: &KeyCode) {
        self.pressed_keys.remove(key);        
    }

    /// Returns true if the specified key is currently being held down.
    ///
    /// # Arguments
    /// * `key` - The key code to query.
    ///
    /// # Returns
    /// `true` if the key is pressed, otherwise `false`.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Returns the current accumulated mouse delta since the last reset.
    ///
    /// # Returns
    /// A `MouseDelta` struct containing horizontal and vertical movement.
    pub fn mouse_delta(&self) -> MouseDelta {
        self.mouse_delta.clone()
    }
}
