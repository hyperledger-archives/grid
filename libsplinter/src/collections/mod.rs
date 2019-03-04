// Copyright 2018 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::hash_map::{Iter, Keys, Values};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BiHashMap<K: Hash + Eq, V: Hash + Eq> {
    kv_hash_map: HashMap<K, V>,
    vk_hash_map: HashMap<V, K>,
}

impl<K: Hash + Eq, V: Hash + Eq> BiHashMap<K, V>
where
    K: std::clone::Clone,
    V: std::clone::Clone,
{
    pub fn new() -> Self {
        BiHashMap {
            kv_hash_map: HashMap::new(),
            vk_hash_map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        BiHashMap {
            kv_hash_map: HashMap::with_capacity(capacity),
            vk_hash_map: HashMap::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        // both maps should have the same capacity
        self.kv_hash_map.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.kv_hash_map.reserve(additional);
        self.vk_hash_map.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.kv_hash_map.shrink_to_fit();
        self.vk_hash_map.shrink_to_fit();
    }

    pub fn keys(&self) -> Keys<K, V> {
        self.kv_hash_map.keys()
    }

    pub fn values(&self) -> Values<K, V> {
        self.kv_hash_map.values()
    }

    pub fn iter_by_keys(&self) -> Iter<K, V> {
        self.kv_hash_map.iter()
    }

    pub fn iter_by_values(&self) -> Iter<V, K> {
        self.vk_hash_map.iter()
    }

    pub fn len(&self) -> usize {
        // both maps should be the same size
        self.kv_hash_map.len()
    }

    pub fn is_empty(&self) -> bool {
        // both maps will be empty or not
        self.kv_hash_map.is_empty()
    }

    pub fn clear(&mut self) {
        self.kv_hash_map.clear();
        self.vk_hash_map.clear();
    }

    pub fn get_by_key(&self, key: &K) -> Option<&V> {
        self.kv_hash_map.get(key)
    }

    pub fn get_by_value(&self, value: &V) -> Option<&K> {
        self.vk_hash_map.get(value)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.kv_hash_map.contains_key(key)
    }

    pub fn contains_value(&self, value: &V) -> bool {
        self.vk_hash_map.contains_key(value)
    }

    // return any overridden values, always in (key, value) format
    pub fn insert(&mut self, key: K, value: V) -> (Option<K>, Option<V>) {
        let old_value = self.kv_hash_map.insert(key.clone(), value.clone());
        let old_key = self.vk_hash_map.insert(value, key);
        (old_key, old_value)
    }

    // If the key is in the map, the removed key and value is returned otherwise None
    pub fn remove_by_key(&mut self, key: &K) -> Option<(K, V)> {
        let value = self.kv_hash_map.remove(key);
        if let Some(value) = value {
            let key = self.vk_hash_map.remove(&value);
            if let Some(key) = key {
                return Some((key, value));
            }
        }
        None
    }

    // If the value is in the map, the removed key and value is returned otherwise None
    pub fn remove_by_value(&mut self, value: &V) -> Option<(K, V)> {
        let key = self.vk_hash_map.remove(value);
        if let Some(key) = key {
            let value = self.kv_hash_map.remove(&key);
            if let Some(value) = value {
                return Some((key, value));
            }
        }
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_capacity() {
        let map: BiHashMap<String, usize> = BiHashMap::new();
        let capacity = map.capacity();
        assert_eq!(capacity, 0);

        let map_with_capacity: BiHashMap<String, usize> = BiHashMap::with_capacity(5);
        let capacity = map_with_capacity.capacity();
        assert!(capacity >= 5);
    }

    #[test]
    fn test_reserve() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        let capacity = map.capacity();
        assert_eq!(capacity, 0);

        map.reserve(5);
        let capacity = map.capacity();
        assert!(capacity >= 5);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut map: BiHashMap<String, usize> = BiHashMap::with_capacity(100);
        let capacity = map.capacity();
        assert!(capacity >= 100);

        map.shrink_to_fit();
        let capacity = map.capacity();
        assert_eq!(capacity, 0);
    }

    #[test]
    fn test_insert() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        assert_eq!((None, None), map.insert("ONE".to_string(), 1));
        assert_eq!(
            (Some("ONE".to_string()), Some(1)),
            map.insert("ONE".to_string(), 1)
        );
        assert_eq!(
            (Some("ONE".to_string()), None),
            map.insert("TWO".to_string(), 1)
        );
        assert_eq!((None, Some(1)), map.insert("ONE".to_string(), 3));
    }

    #[test]
    fn test_keys_and_values() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        map.insert("ONE".to_string(), 1);
        map.insert("TWO".to_string(), 2);
        map.insert("THREE".to_string(), 3);

        let mut keys: Vec<String> = map.keys().map(|key| key.to_string()).collect();
        keys.sort();
        assert_eq!(
            keys,
            ["ONE".to_string(), "THREE".to_string(), "TWO".to_string()]
        );

        let mut values: Vec<usize> = map.values().map(|value| value.clone()).collect();
        values.sort();
        assert_eq!(values, [1, 2, 3])
    }

    #[test]
    fn test_iter_keys_and_values() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        map.insert("ONE".to_string(), 1);
        map.insert("TWO".to_string(), 2);
        map.insert("THREE".to_string(), 3);
        let keys = vec!["ONE".to_string(), "THREE".to_string(), "TWO".to_string()];
        let values = vec![1, 2, 3];

        for (key, value) in map.iter_by_keys() {
            assert!(keys.contains(key));
            assert!(values.contains(value));
        }

        for (value, key) in map.iter_by_values() {
            assert!(keys.contains(key));
            assert!(values.contains(value));
        }
    }

    #[test]
    fn test_clear_and_is_empty() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        map.insert("ONE".to_string(), 1);
        map.insert("TWO".to_string(), 2);
        map.insert("THREE".to_string(), 3);

        assert_eq!(map.len(), 3);
        assert!(!map.is_empty());

        map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_get() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        map.insert("ONE".to_string(), 1);
        map.insert("TWO".to_string(), 2);
        map.insert("THREE".to_string(), 3);

        assert_eq!(map.get_by_key(&"ONE".to_string()), Some(&1));
        assert_eq!(map.get_by_key(&"TWO".to_string()), Some(&2));
        assert_eq!(map.get_by_key(&"THREE".to_string()), Some(&3));
        assert_eq!(map.get_by_key(&"FOUR".to_string()), None);

        assert_eq!(map.get_by_value(&1), Some(&"ONE".to_string()));
        assert_eq!(map.get_by_value(&2), Some(&"TWO".to_string()));
        assert_eq!(map.get_by_value(&3), Some(&"THREE".to_string()));
        assert_eq!(map.get_by_value(&4), None);
    }

    #[test]
    fn test_contains_key_and_value() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        map.insert("ONE".to_string(), 1);

        assert!(map.contains_key(&"ONE".to_string()));
        assert!(map.contains_value(&1));

        assert!(!map.contains_key(&"TWO".to_string()));
        assert!(!map.contains_value(&2));
    }

    #[test]
    fn test_removes() {
        let mut map: BiHashMap<String, usize> = BiHashMap::new();
        map.insert("ONE".to_string(), 1);
        map.insert("TWO".to_string(), 2);
        map.insert("THREE".to_string(), 3);

        let removed = map.remove_by_key(&"ONE".to_string());
        assert_eq!(removed, Some(("ONE".to_string(), 1)));

        let removed = map.remove_by_key(&"ONE".to_string());
        assert_eq!(removed, None);

        let removed = map.remove_by_value(&2);
        assert_eq!(removed, Some(("TWO".to_string(), 2)));

        let removed = map.remove_by_value(&2);
        assert_eq!(removed, None);
    }
}
