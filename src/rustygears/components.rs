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

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

type BoxedComponent = Arc<RwLock<Box<dyn Any + Send + Sync>>>;

#[derive(Clone)]
pub struct ComponentMap {
    components: HashMap<TypeId, BoxedComponent>,
}

impl ComponentMap {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        let component = Arc::new(RwLock::new(Box::new(value) as Box<dyn Any + Send + Sync>));
        self.components.insert(type_id, component);
    }

    pub fn get<T: 'static + Send + Sync>(&self) -> Result<ReadGuardWrapper<T>, String> {
        let type_id = TypeId::of::<T>();
        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component {} not found", std::any::type_name::<T>()))?;
        
        let guard = arc.read()
            .map_err(|_| "Component is poisoned".to_string())?;

        Ok(ReadGuardWrapper {
            guard,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn get_mut<T: 'static + Send + Sync>(&self) -> Result<WriteGuardWrapper<T>, String> {
        let type_id = TypeId::of::<T>();
        let arc = self.components.get(&type_id)
            .ok_or_else(|| format!("Component {} not found", std::any::type_name::<T>()))?;
        
        let guard = arc.write()
            .map_err(|_| "Component is poisoned".to_string())?;

        Ok(WriteGuardWrapper {
            guard,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn with<T: 'static + Send + Sync, R>(
        &self,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, String> {
        let mut guard = self.get_mut::<T>()?;
        Ok(f(&mut *guard))
    }
}

// Send + Sync wrapper for read access
pub struct ReadGuardWrapper<'a, T> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

// SAFETY: The guard is Send because:
// 1. The Box<dyn Any + Send + Sync> is Send (since its contents are Send)
// 2. We're only giving access to the T which is Send
unsafe impl<'a, T: Send> Send for ReadGuardWrapper<'a, T> {}

impl<'a, T: 'static> Deref for ReadGuardWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref::<T>()
            .expect("TypeId matched but downcast failed")
    }
}

// Send + Sync wrapper for write access
pub struct WriteGuardWrapper<'a, T> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

// SAFETY: The guard is Send because:
// 1. The Box<dyn Any + Send + Sync> is Send (since its contents are Send)
// 2. We're only giving access to the T which is Send
unsafe impl<'a, T: Send> Send for WriteGuardWrapper<'a, T> {}

impl<'a, T: 'static> Deref for WriteGuardWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.downcast_ref::<T>()
            .expect("TypeId matched but downcast failed")
    }
}

impl<'a, T: 'static> DerefMut for WriteGuardWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.downcast_mut::<T>()
            .expect("TypeId matched but downcast failed")
    }
}
