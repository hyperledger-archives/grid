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

//! Storage trait for syncing writes to an object to a backing store
//!
//! Hands out {read, write} RAII-guarded references to an object, and ensures
//! that when the reference drops, any changes to the object are persisted to
//! the selected storage.

pub mod state;
pub mod yaml;

use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;
use serde::Serialize;

pub use self::yaml::YamlStorage;

/// RAII structure used to allow read access to state object
///
/// This guard allows avoiding unnecessary syncing if you just need read
/// access to the state object.
pub trait StorageReadGuard<'a, T: Sized>: Deref<Target = T> {}

/// RAII structure used to allow write access to state object
///
/// This guard will ensure that any changes to an object are persisted to
/// a backing store when this is Dropped.
pub trait StorageWriteGuard<'a, T: Sized>: DerefMut<Target = T> {}

/// Storage wrapper that ensures that changes to an object are persisted to a backing store
///
/// Achieves this by handing out RAII-guarded references to the underlying data, that ensure
/// persistence when they get Dropped.
pub trait Storage {
    type S;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, Self::S, Target = Self::S> + 'a>;
    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, Self::S, Target = Self::S> + 'a>;
}

/// Given a location string, returns the appropriate storage
///
/// Accepts `"memory"` or `"disk+/path/to/file"` as location values
pub fn get_storage<'a, T: Sized + Serialize + DeserializeOwned + 'a, F: Fn() -> T>(
    location: &str,
    default: F,
) -> Result<Box<dyn Storage<S = T> + 'a>, String> {
    if location.ends_with(".yaml") {
        Ok(Box::new(YamlStorage::new(location, default).unwrap()) as Box<Storage<S = T>>)
    } else {
        Err(format!("Unknown state location type: {}", location))
    }
}

#[cfg(test)]
mod tests {
    use super::YamlStorage;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_read_guard() {
        let temp_dir = TempDir::new("test_read_guard").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let filename = temp_dir_path.to_str().unwrap().to_string();

        println!("{}", filename);
        let storage = YamlStorage::new(filename.clone(), || 1).unwrap();
        let val = storage.read();
        let other = storage.read();
        assert_eq!(**val, 1);
        assert_eq!(**other, 1);
    }

    #[test]
    // Ensures that data is persisted between object lifetimes
    fn test_disk_persistence() {
        let temp_dir = TempDir::new("test_disk_persistence").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let filename = temp_dir_path.to_str().unwrap().to_string();

        {
            let mut storage = YamlStorage::new(&filename[..], || 0).unwrap();
            let mut val = storage.write();
            **val = 5;
            assert_eq!(**val, 5);
        }
        let storage = YamlStorage::new(&filename[..], || 0).unwrap();
        let val = storage.read();
        assert_eq!(**val, 5);
    }

    #[test]
    // Ensure we don't overwrite longer data with shorter data, and get a mixture of the two
    fn test_truncation() {
        let temp_dir = TempDir::new("test_truncation").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let filename = temp_dir_path.to_str().unwrap().to_string();

        {
            let storage = YamlStorage::new(&filename[..], || 500).unwrap();
            let val = storage.read();
            assert_eq!(**val, 500);
        }

        {
            let mut storage = YamlStorage::new(&filename[..], || 0).unwrap();
            let mut val = storage.write();
            assert_eq!(**val, 500);
            **val = 2;
            assert_eq!(**val, 2);
        }

        let storage = YamlStorage::new(&filename[..], || 0).unwrap();
        let val = storage.read();
        assert_eq!(**val, 2);
    }

    #[test]
    fn test_write_guard() {
        let temp_dir = TempDir::new("test_write_guard").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let filename = temp_dir_path.to_str().unwrap().to_string();

        {
            let mut storage = YamlStorage::new(&filename[..], || 1).unwrap();
            let mut val = storage.write();
            assert_eq!(**val, 1);
            **val = 5;
            assert_eq!(**val, 5);
        }

        {
            let mut storage = YamlStorage::new(&filename[..], || 1).unwrap();
            let mut val = storage.write();
            assert_eq!(**val, 5);
            **val = 64;
            assert_eq!(**val, 64);
        }
    }

    #[test]
    fn test_get_storage() {
        let temp_dir = TempDir::new("test_get_storage").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let filename = temp_dir_path.to_str().unwrap().to_string();

        let mut yaml = get_storage(&format!("{}.yaml", filename), || 1).unwrap();

        assert_eq!(**yaml.read(), 1);

        {
            let mut val = yaml.write();
            **val = 128;
        }

        assert_eq!(**yaml.read(), 128);

        if let Ok(_) = get_storage("not_yaml.file", || 1) {
            panic!("get_storage did not fail when given a bad file type");
        }
    }
}
