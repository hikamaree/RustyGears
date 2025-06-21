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

mod camera;
mod graphics;
mod physics;
mod time;
mod rustygears;
mod system;
mod winit;
mod scene;
mod ecs;
mod render;
mod input;
mod terminal;
mod logs;

pub mod math {
    pub use cgmath::*;
}

use physics::*;

pub use physics::{
    PhysicsWorld,
    RigidBody,
};

pub use graphics::*;

pub use time::Time;
pub use rustygears::*;
pub use winit::*;
pub use system::*;
pub use camera::*;
pub use scene::*;
pub use ecs::*;
pub use render::*;
pub use input::*;
pub use terminal::*;
pub use logs::*;

pub use crossbeam::channel::Receiver;
pub use crossbeam::channel::Sender;
