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

use std::any::Any;
use crate::GameView;
use egui::Context;

/// A trait representing a renderable GUI panel or component.
///
/// This trait is implemented by any user-defined GUI widget or panel that wants to participate
/// in the rendering cycle of the UI. It provides a method to draw itself using an `egui::Context`,
/// along with optional downcasting support through `as_any` and `as_any_mut`.
///
/// GUI elements are typically stored as trait objects (`Box<dyn Gui + Send + Sync>`) and rendered
/// each frame by the main UI system.
pub trait Gui: Any + Send + Sync {
    /// Renders the GUI component.
    ///
    /// Called once per frame during the UI pass. Receives a reference to the immutable game state
    /// and the `egui` context for creating widgets and windows.
    ///
    /// # Arguments
    /// * `game` - A view of the current game state (`GameView`), allowing read-only access to components.
    /// * `context` - The `egui::Context` used to build the UI for this frame.
    fn render_gui(&mut self, game: &GameView, context: &Context);

    /// Returns a reference to the current object as `dyn Any` for downcasting.
    ///
    /// Useful when you need to recover the original concrete type behind the trait object.
    fn as_any(&self) -> &dyn Any
    where
        Self: 'static + Sized,
    {
        self
    }

    /// Returns a mutable reference to the current object as `dyn Any` for downcasting.
    ///
    /// Allows mutation of the concrete type after a successful downcast.
    fn as_any_mut(&mut self) -> &mut dyn Any
    where
        Self: 'static + Sized,
    {
        self
    }
}
