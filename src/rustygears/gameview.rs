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

use crate::Graphics;
use crate::Time;
use crate::Scene;

/// A read-only view of the current game state, provided to [`Gear`]s when handling events.
///
/// `GameView` allows gears to inspect parts of the game world such as other gears, graphics state, timing information,
/// and the scene. It provides safe access without allowing direct mutation of the game.

pub struct GameView {
    /// A reference to the graphics context, used for rendering-related information or operations.
    pub graphics: Graphics,

    /// A reference to the time subsystem, providing timing and delta-time information.
    pub time: Time,

    /// A reference to the current scene, which may contain entities or spatial information.
    pub scene: Scene,
}
