/*
 * Copyright 2018 Bitwise IO, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

//! Memory-backed persistence wrapper
//!
//! Useful when a Storage impl is required, but you don't actually need to
//! persist the wrapped object.

use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Storage, StorageReadGuard, StorageWriteGuard};

/// Memory-backed read guard
#[derive(Debug)]
pub struct MemStorageReadGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    storage: &'a MemStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> MemStorageReadGuard<'a, T> {
    fn new(storage: &'a MemStorage<T>) -> Self {
        Self { storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for MemStorageReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for MemStorageReadGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned> StorageReadGuard<'a, T>
    for MemStorageReadGuard<'a, T>
{}

/// Memory-backed write guard
#[derive(Debug)]
pub struct MemStorageWriteGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    storage: &'a mut MemStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> MemStorageWriteGuard<'a, T> {
    fn new(storage: &'a mut MemStorage<T>) -> Self {
        Self { storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for MemStorageWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.storage.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> DerefMut for MemStorageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for MemStorageWriteGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned> StorageWriteGuard<'a, T>
    for MemStorageWriteGuard<'a, T>
{}

/// Memory-backed RAII-guarded Storage implementation
///
/// Can be used when actual persistence isn't required
#[derive(Debug)]
pub struct MemStorage<T: Serialize + DeserializeOwned> {
    data: T,
}

impl<T: Serialize + DeserializeOwned> MemStorage<T> {
    pub fn new<F: Fn() -> T>(default: F) -> Result<Self, String> {
        Ok(Self { data: default() })
    }
}

impl<T: Serialize + DeserializeOwned + fmt::Display> fmt::Display for MemStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).data.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned> Storage for MemStorage<T> {
    type S = T;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, T, Target = T> + 'a> {
        Box::new(MemStorageReadGuard::new(self))
    }

    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, T, Target = T> + 'a> {
        Box::new(MemStorageWriteGuard::new(self))
    }
}
