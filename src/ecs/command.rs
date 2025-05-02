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

use crate::Game;

/// A trait that represents a game command that can be executed on a [`Game`] instance.
///
/// Implementors of this trait define logic to modify game state.

pub trait Command: Send + Sync {

    /// Applies the command to the given game instance.
    ///
    /// This method consumes the boxed command.
    ///
    /// # Parameters
    /// - `game`: A mutable reference to the [`Game`] instance to modify.

    fn apply(self: Box<Self>, game: &mut Game);
}
