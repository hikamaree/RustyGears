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

use crate::Entity;
use crate::Component;
use crate::World;

pub struct EntityBuilder<'a> {
    pub(crate) world: &'a mut World,
    pub(crate) entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<T: Component>(self, component: T) -> Self {
        self.world.insert(self.entity, component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
