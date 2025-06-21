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

use std::any::TypeId;
use std::any::Any;
use std::collections::HashMap;

/// A read-only view of the current game state, provided to [`Gear`]s when handling events.
///
/// `GameView` allows gears to inspect parts of the game world such as other gears, graphics state, timing information,
/// and the scene. It provides safe access without allowing direct mutation of the game.
pub struct GameView {
    pub(crate) components: HashMap<TypeId, &'static (dyn Any + Send + Sync)>,
}

impl GameView {
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.downcast_ref::<T>()
    }
}
