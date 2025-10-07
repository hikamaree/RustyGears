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

use crate::Component;
use crate::World;

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a unique identifier for an entity within the ECS (Entity-Component System) world.
///
/// An `Entity` is the fundamental unit of identity in the ECS architecture.  
/// Each entity is represented by a unique `u64` ID, which serves as a key
/// to associate one or more *components* within the [`World`].
///
/// Entities themselves are *lightweight* — they do not store any data directly.
/// Instead, data is stored in components, which are inserted and managed
/// through the ECS world.
///
/// ### Key Properties
/// - Entities are uniquely identified by a monotonically increasing counter (`ID_COUNTER`).
/// - The special [`Entity::NULL`] constant represents a *non-existent* or *invalid* entity.
/// - Entities can be compared (`==`, `!=`), hashed, and copied freely.
///
/// ### Typical Usage
/// ```
/// let e = Entity::new(); // create a new unique entity
/// println!("Entity ID: {}", e.id());
///
/// // Example of inserting a component (pseudo-code)
/// e.insert(Position { x: 5.0, y: 3.0 });
/// ```
///
/// ### ECS Context
/// In an ECS world:
/// - **Entity:** identifier (like a handle)
/// - **Component:** data (like position, velocity, health)
/// - **System:** logic that processes entities with specific components
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    id: u64
}

impl Entity {
    /// Represents a null (invalid) entity handle.
    ///
    /// `Entity::NULL` is used to represent an uninitialized, destroyed, or placeholder entity.
    /// It is guaranteed to have an ID of `0`.
    ///
    /// ### Example
    /// ```
    /// let e = Entity::NULL;
    /// assert_eq!(e.id(), 0);
    /// ```
    pub const NULL: Entity = Entity { id: 0 };

    /// Creates a new unique entity with an automatically assigned ID.
    ///
    /// Each call increments a global atomic counter to ensure thread-safe
    /// uniqueness across all entities, even if they are created from
    /// multiple threads concurrently.
    ///
    /// ### Example
    /// ```
    /// let e1 = Entity::new();
    /// let e2 = Entity::new();
    /// assert_ne!(e1, e2);
    /// ```
    pub fn new() -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Returns an entity instance for a given ID.
    ///
    /// Useful for reconstructing entities from serialized or pre-known IDs.
    ///
    /// **Note:** This does not check whether the entity actually exists in the world.
    ///
    /// ### Example
    /// ```
    /// let e = Entity::get(42);
    /// assert_eq!(e.id(), 42);
    /// ```
    pub fn get(id: u64) -> Entity {
        Self { id }
    }

    /// Returns the numeric ID of this entity.
    ///
    /// ### Example
    /// ```
    /// let e = Entity::new();
    /// println!("Entity ID = {}", e.id());
    /// ```
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Inserts a component into the world for this entity.
    ///
    /// The component is associated with this entity and will be available
    /// for systems and gears to access during the next update cycle.
    ///
    /// This method schedules the insertion to occur safely within the game's
    /// main update loop.
    ///
    /// # Example
    /// ```
    /// let entity = Entity::new();
    /// entity.insert(Health { value: 100 });
    /// ```
    pub fn insert<T: Component>(self, component: T) -> Self {
        crate::send_command(crate::CommandFunction {
            run: Box::new(move |game| {
                let Ok(mut scene) = game.components.get_mut::<crate::WorldScene>() else { return; };
                scene.world.insert(self, component);
            }),
        });
        self
    }
}

/// Builder pattern for creating entities with multiple components.
///
/// Instead of inserting each component separately with [`Entity::insert`],
/// you can use `EntityBuilder` to chain multiple `.with()` calls and
/// finalize creation with `.build()`.
///
/// ### Example
/// ```
/// let entity = EntityBuilder {
///     world: &mut world,
///     entity: Entity::new(),
/// }
/// .with(Position { x: 0.0, y: 0.0 })
/// .with(Velocity { dx: 1.0, dy: 0.0 })
/// .build();
/// ```
pub struct EntityBuilder<'a> {
    pub world: &'a mut World,
    pub entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    /// Inserts a new component into the world for this entity.
    ///
    /// This can be chained to add multiple components in one call sequence.
    ///
    /// ### Example
    /// ```
    /// builder.with(Position { x: 10.0, y: 5.0 })
    ///        .with(Health { hp: 100 });
    /// ```
    pub fn with<T: Component>(self, component: T) -> Self {
        self.world.insert(self.entity, component);
        self
    }

    /// Finalizes the builder and returns the constructed entity.
    ///
    /// ### Example
    /// ```
    /// let e = builder.build();
    /// ```
    pub fn build(self) -> Entity {
        self.entity
    }
}
