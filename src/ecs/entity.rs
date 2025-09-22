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

// use crate::Entity;
use crate::Component;
use crate::World;

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a unique entity within the ECS world.
///
/// Each entity is identified by a unique `u32` ID. Entities themselves do not store any data;
/// all meaningful data is stored in components associated with the entity through the `World`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    id: u64
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn get(id: u64) -> Entity {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

pub struct EntityBuilder<'a> {
    pub world: &'a mut World,
    pub entity: Entity,
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
