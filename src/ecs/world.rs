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

use std::collections::HashMap;

use std::any::TypeId;
use std::any::Any;

/// Represents a unique entity within the ECS world.
///
/// Each entity is identified by a unique `u32` ID. Entities themselves do not store any data;
/// all meaningful data is stored in components associated with the entity through the `World`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(u32);

/// Marker trait for all types that can be used as components in the ECS.
///
/// Components are plain data types (typically `struct`s) that describe various aspects
/// or behaviors of an entity. They must be:
/// - `Send` and `Sync`: to allow safe access across threads.
/// - `Clone`: to support duplication during certain operations like snapshotting.
/// - `'static`: to ensure the type does not contain non-static references.
pub trait Component: Send + Sync + Clone + 'static {}
impl<T: Send + Sync + Clone + 'static> Component for T {}

/// Internal storage structure for managing all components in the ECS.
///
/// This structure maps `TypeId`s of components to dynamically typed boxed vectors.
/// Each component type has its own separate vector of optional values, indexed by entity ID.
/// The value at a given index corresponds to the component (if any) associated with that entity.
#[derive(Default)]
struct ComponentStorage {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComponentStorage {
    /// Creates a new, empty `ComponentStorage`.
    ///
    /// Used internally by the `World` to initialize its component store.
    fn new() -> Self {
        Self { data: HashMap::new() }
    }

    /// Inserts a pre-allocated component vector for a specific component type.
    ///
    /// # Parameters
    /// - `components`: A vector where each entry is either `Some(component)` or `None`,
    ///   representing the presence or absence of that component on a specific entity index.
    ///
    /// This is primarily used to allocate component storage up-front when a new type is inserted
    /// into the ECS for the first time.
    fn insert<T: Component>(&mut self, components: Vec<Option<T>>) {
        self.data.insert(TypeId::of::<T>(), Box::new(components));
    }

    /// Retrieves an immutable reference to the component vector for type `T`.
    ///
    /// Returns `None` if the component type `T` has never been inserted.
    fn get_slice<T: Component>(&self) -> Option<&Vec<Option<T>>> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Vec<Option<T>>>())
    }

    /// Retrieves a mutable reference to the component vector for type `T`.
    ///
    /// Returns `None` if the component type `T` has never been inserted.
    fn get_slice_mut<T: Component>(&mut self) -> Option<&mut Vec<Option<T>>> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<Vec<Option<T>>>())
    }
}

/// Represents an Entity-Component-System (ECS) world.
///
/// The `World` manages entity identifiers and component storage. Each component type
/// is stored in a type-erased vector that maps entity IDs to component instances.
///
/// This ECS implementation supports:
/// - Spawning entities
/// - Adding, removing, and accessing components
/// - Querying entities with 1 to 4 component types
#[derive(Default)]
pub struct World {
    next_id: u32,
    storage: ComponentStorage,
    capacity: usize,
}

impl World {
    /// Creates a new `World` instance with default capacity.
    ///
    /// # Returns
    /// A new, empty `World`.
    pub fn new() -> Self {
        Self {
            next_id: 0,
            storage: ComponentStorage::new(),
            capacity: 10000000,
        }
    }

    /// Creates a new entity and returns its unique ID.
    ///
    /// This ID can then be used to attach components.
    ///
    /// # Returns
    /// A new `Entity`.
    pub fn spawn(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        Entity(id)
    }

    /// Inserts a component for a specific entity.
    ///
    /// Automatically expands storage for the component type if it doesn't exist or isn't large enough.
    ///
    /// # Parameters
    /// - `entity`: The entity ID.
    /// - `component`: The component instance to attach.
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        if self.storage.get_slice_mut::<T>().is_none() {
            self.storage.insert::<T>(vec![None; self.capacity]);
        }

        let Some(vec) = self.storage.get_slice_mut::<T>() else {
            return;
        };

        if entity.0 as usize >= vec.len() {
            vec.resize_with(entity.0 as usize + 1, || None);
        }

        vec[entity.0 as usize] = Some(component);
    }

    /// Retrieves an immutable reference to a component for the given entity.
    ///
    /// # Returns
    /// - `Some(&T)` if the component exists.
    /// - `None` otherwise.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.storage.get_slice::<T>()
            .and_then(|vec| vec.get(entity.0 as usize).and_then(|opt| opt.as_ref()))
    }

    /// Retrieves a mutable reference to a component for the given entity.
    ///
    /// # Returns
    /// - `Some(&mut T)` if the component exists.
    /// - `None` otherwise.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.storage.get_slice_mut::<T>()
            .and_then(|vec| vec.get_mut(entity.0 as usize).and_then(|opt| opt.as_mut()))
    }

    /// Removes a component of a specific type from an entity.
    ///
    /// # Parameters
    /// - `entity`: The entity ID.
    pub fn remove<T: Component>(&mut self, entity: Entity) {
        if let Some(vec) = self.storage.get_slice_mut::<T>() {
            if let Some(slot) = vec.get_mut(entity.0 as usize) {
                *slot = None;
            }
        }
    }

    /// Checks whether the given entity has a component of type `T`.
    ///
    /// # Returns
    /// `true` if the component exists, `false` otherwise.
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.get::<T>(entity).is_some()
    }

    /// Queries all entities that have a component of type `T`.
    ///
    /// # Returns
    /// A vector of `(Entity, &T)` pairs for all matching entities.
    pub fn query1<'a, T: Component>(&'a self) -> Vec<(Entity, &'a T)> {
        match self.storage.get_slice::<T>() {
            Some(slice) => slice.iter().enumerate()
                .filter_map(|(i, opt)| opt.as_ref().map(|c| (Entity(i as u32), c)))
                .collect(),
            None => vec![],
        }
    }

    /// Queries all entities that have both component types `A` and `B`.
    ///
    /// # Returns
    /// A vector of `(Entity, &A, &B)` for all matching entities.
    pub fn query2<'a, A: Component, B: Component>(&'a self) -> Vec<(Entity, &'a A, &'a B)> {
        let slice_a = match self.storage.get_slice::<A>() {
            Some(s) => s,
            None => return vec![],
        };
        let slice_b = match self.storage.get_slice::<B>() {
            Some(s) => s,
            None => return vec![],
        };

        let len = slice_a.len().min(slice_b.len());

        let mut result = Vec::new();
        for i in 0..len {
            if let (Some(a), Some(b)) = (&slice_a[i], &slice_b[i]) {
                result.push((Entity(i as u32), a, b));
            }
        }

        result
    }

    /// Queries all entities that have component types `A`, `B`, and `C`.
    ///
    /// # Returns
    /// A vector of `(Entity, &A, &B, &C)` for all matching entities.
    pub fn query3<'a, A: Component, B: Component, C: Component>(&'a self) -> Vec<(Entity, &'a A, &'a B, &'a C)> {
        let a = self.storage.get_slice::<A>();
        let b = self.storage.get_slice::<B>();
        let c = self.storage.get_slice::<C>();

        match (a, b, c) {
            (Some(a), Some(b), Some(c)) => {
                let len = a.len().min(b.len()).min(c.len());
                let mut result = Vec::new();
                for i in 0..len {
                    if let (Some(aa), Some(bb), Some(cc)) = (&a[i], &b[i], &c[i]) {
                        result.push((Entity(i as u32), aa, bb, cc));
                    }
                }
                result
            }
            _ => vec![],
        }
    }

    /// Queries all entities that have component types `A`, `B`, `C`, and `D`.
    ///
    /// # Returns
    pub fn query4<'a, A: Component, B: Component, C: Component, D: Component>(
        &'a self,
    ) -> Vec<(Entity, &'a A, &'a B, &'a C, &'a D)> {
        let a = self.storage.get_slice::<A>();
        let b = self.storage.get_slice::<B>();
        let c = self.storage.get_slice::<C>();
        let d = self.storage.get_slice::<D>();

        match (a, b, c, d) {
            (Some(a), Some(b), Some(c), Some(d)) => {
                let len = a.len().min(b.len()).min(c.len()).min(d.len());
                let mut result = Vec::new();
                for i in 0..len {
                    if let (Some(aa), Some(bb), Some(cc), Some(dd)) = (&a[i], &b[i], &c[i], &d[i]) {
                        result.push((Entity(i as u32), aa, bb, cc, dd));
                    }
                }
                result
            }
            _ => vec![],
        }
    }

    /// Prints a list of all entities that have a component of type `T`.
    ///
    /// This is primarily useful for debugging purposes.
    ///
    /// # Example Output
    /// ```
    /// Entities:
    /// Entity(0)
    /// Entity(4)
    /// Entity(27)
    /// ```
    pub fn debug_print_entities<T: Component>(&self) {
        if let Some(slice) = self.storage.get_slice::<T>() {
            println!("Entities:");
            for (i, slot) in slice.iter().enumerate() {
                if slot.is_some() {
                    println!("Entity({})", i);
                }
            }
        }
    }
}
