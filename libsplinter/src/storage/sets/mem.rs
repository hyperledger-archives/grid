// Copyright 2019 Cargill Incorporated
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

//! In-memory implementations of the DurableSet traits.

use std::borrow::Borrow;
use std::cmp::Ord;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use super::{DurableOrderedSet, DurableRange, DurableSet, DurableSetError};

/// An in-memory, DurableSet, backed by a HashSet.
///
/// This set is unbounded.
#[derive(Default)]
pub struct DurableHashSet<V: Hash + Eq> {
    inner: Arc<Mutex<HashSet<V>>>,
}

impl<V: Send + Hash + Eq> DurableHashSet<V> {
    /// Constructs a new DurableHashSet.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl<V: Send + Hash + Eq + Clone> DurableSet for DurableHashSet<V> {
    type Item = V;

    /// Add an item to the set.
    fn add(&mut self, item: Self::Item) -> Result<(), DurableSetError> {
        self.inner
            .lock()
            .map_err(|_| {
                DurableSetError::new("Poisoned lock error occurred while attempting to insert item")
            })?
            .insert(item);

        Ok(())
    }

    /// Remove an item to the set.
    fn remove(&mut self, item: &Self::Item) -> Result<Option<Self::Item>, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new("Poisoned lock error occurred while attempting to remove item")
            })?
            .take(item))
    }

    fn iter<'a>(&'a self) -> Result<Box<(dyn Iterator<Item = Self::Item> + 'a)>, DurableSetError> {
        Ok(Box::new(SnapShotIter {
            snapshot: self
                .inner
                .lock()
                .map_err(|_| {
                    DurableSetError::new("Poisoned lock error occurred while attempting to iterate")
                })?
                .iter()
                .cloned()
                .collect(),
        }))
    }

    fn contains(&self, item: &Self::Item) -> Result<bool, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to check if the set contains \
                     an item",
                )
            })?
            .contains(item))
    }

    fn len(&self) -> Result<u64, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to return the length of the set",
                )
            })?
            .len() as u64)
    }
}

struct SnapShotIter<V: Send + Clone> {
    snapshot: std::collections::VecDeque<V>,
}

impl<V: Send + Clone> Iterator for SnapShotIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.snapshot.pop_front()
    }
}

/// An in-memory, DurableOrderedSet, backed by a BTreeSet.
///
/// This set is bounded, where in it will drop the first item in the set, based on the natural
/// order of the items stored.
#[derive(Default, Clone)]
pub struct DurableBTreeSet<V: Ord + Send> {
    inner: Arc<Mutex<BTreeSet<V>>>,
    bound: usize,
}

impl<V: Ord + Send> DurableBTreeSet<V> {
    pub fn new_boxed<Index>() -> Box<dyn DurableOrderedSet<V, Index>>
    where
        Index: Ord + Send + Clone,
        V: Send + Ord + Borrow<Index> + Clone + 'static,
    {
        Box::new(Self {
            inner: Arc::new(Mutex::new(BTreeSet::new())),
            bound: std::usize::MAX,
        })
    }

    pub fn new_boxed_with_bound<Index>(bound: NonZeroUsize) -> Box<dyn DurableOrderedSet<V, Index>>
    where
        Index: Ord + Send + Clone,
        V: Send + Ord + Borrow<Index> + Clone + 'static,
    {
        Box::new(Self {
            inner: Arc::new(Mutex::new(BTreeSet::new())),
            bound: bound.get(),
        })
    }
}

impl<V> DurableSet for DurableBTreeSet<V>
where
    V: Send + Ord + Clone,
{
    type Item = V;

    /// Add an item to the set.
    fn add(&mut self, item: Self::Item) -> Result<(), DurableSetError> {
        let mut set = self.inner.lock().map_err(|_| {
            DurableSetError::new("Poisoned lock error occurred while attempting to insert item")
        })?;

        if set.len() == self.bound {
            let rm_lowest = set.iter().next().cloned().unwrap();
            set.remove(&rm_lowest);
        }

        set.insert(item);

        Ok(())
    }

    /// Remove an item to the set.
    fn remove(&mut self, item: &Self::Item) -> Result<Option<Self::Item>, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new("Poisoned lock error occurred while attempting to remove item")
            })?
            .take(item))
    }

    fn iter<'a>(&'a self) -> Result<Box<(dyn Iterator<Item = Self::Item> + 'a)>, DurableSetError> {
        Ok(Box::new(SnapShotIter {
            snapshot: self
                .inner
                .lock()
                .map_err(|_| {
                    DurableSetError::new("Poisoned lock error occurred while attempting to iterate")
                })?
                .iter()
                .cloned()
                .collect(),
        }))
    }

    fn contains(&self, item: &Self::Item) -> Result<bool, DurableSetError> {
        Ok(self.inner
            .lock()
            .map_err(|_| DurableSetError::new("Poisoned lock error occurred while attempting to check if the set contains an item"))?
            .contains(item))
    }

    fn len(&self) -> Result<u64, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to return the length of the set",
                )
            })?
            .len() as u64)
    }
}

impl<V, Index> DurableOrderedSet<V, Index> for DurableBTreeSet<V>
where
    Index: Ord + Send,
    V: Send + Ord + Borrow<Index> + Clone + 'static,
{
    fn get_by_index(&self, index_value: &Index) -> Result<Option<Self::Item>, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to retrieve an item by index",
                )
            })?
            .get(index_value)
            .cloned())
    }

    fn contains_by_index(&self, index_value: &Index) -> Result<bool, DurableSetError> {
        Ok(self.inner
            .lock()
            .map_err(|_| DurableSetError::new("Poisoned lock error occurred while attempting to check if the set contains an item"))?
            .contains(index_value))
    }

    /// Returns an iterator over a range
    fn range_iter<'a>(
        &'a self,
        range: DurableRange<&Index>,
    ) -> Result<Box<(dyn Iterator<Item = Self::Item> + 'a)>, DurableSetError> {
        Ok(Box::new(SnapShotIter {
            snapshot: self
                .inner
                .lock()
                .map_err(|_| {
                    DurableSetError::new("Poisoned lock error occurred while attempting to iterate")
                })?
                .range((range.start, range.end))
                .cloned()
                .collect(),
        }))
    }

    fn first(&self) -> Result<Option<Self::Item>, DurableSetError> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to get first item",
                )
            })?
            .iter()
            .next()
            .cloned())
    }

    fn last(&self) -> Result<Option<Self::Item>, DurableSetError> {
        // We can use the last function on the BTreeSet's iterator, as it is O(1)
        Ok(self
            .inner
            .lock()
            .map_err(|_| {
                DurableSetError::new(
                    "Poisoned lock error occurred while attempting to get last item",
                )
            })?
            .iter()
            .last()
            .cloned())
    }

    fn clone_boxed_ordered_set(&self) -> Box<dyn DurableOrderedSet<V, Index>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Test that DurableHashSet add will insert the items provided and verify that they are in the
    /// set by iterating over all the members
    #[test]
    fn test_hash_add() {
        let mut hash_set = DurableHashSet::new();
        assert!(hash_set.is_empty().expect("Unable to get is_empty"));
        hash_set
            .add("hello".to_string())
            .expect("unable to add value");
        hash_set
            .add("bonjour".to_string())
            .expect("unable to add value");
        hash_set
            .add("guten tag".to_string())
            .expect("unable to add value");

        assert!(!hash_set.is_empty().expect("Unable to get is_empty"));
        assert_eq!(3, hash_set.len().expect("Unable to get len"));
        let mut contents = hash_set
            .iter()
            .expect("Could not create iterator")
            .collect::<Vec<_>>();

        contents.sort();
        assert_eq!(vec!["bonjour", "guten tag", "hello"], contents);
    }

    /// Using DurableHashSet, insert three items and remove:
    /// a) an non-existent entry
    /// b) an existing item
    /// c) the same item
    /// Verify the removeal by iterating over the remaining items.
    #[test]
    fn test_hash_remove() {
        let mut hash_set = DurableHashSet::new();
        hash_set
            .add("hello".to_string())
            .expect("unable to add value");
        hash_set
            .add("bonjour".to_string())
            .expect("unable to add value");
        hash_set
            .add("guten tag".to_string())
            .expect("unable to add value");

        // Check that a non-existent key returns None
        assert_eq!(
            None,
            hash_set
                .remove(&"goodbye".to_string())
                .expect("Unable to remove")
        );
        // Remove a known key
        assert_eq!(
            Some("hello".into()),
            hash_set
                .remove(&"hello".to_string())
                .expect("Unable to remove")
        );
        // Check that the key is now removed
        assert_eq!(
            None,
            hash_set
                .remove(&"hello".to_string())
                .expect("Unable to remove")
        );

        let mut contents = hash_set
            .iter()
            .expect("Could not create iterator")
            .collect::<Vec<_>>();

        contents.sort();
        assert_eq!(vec!["bonjour", "guten tag"], contents,);
    }

    /// Using DurableBTreeSet, insert three items, sorted by integer IDS.
    /// - Verify that an item can be retrieved by integer index
    /// - Verify that the items are iterated in order
    #[test]
    fn test_btree_add_int() {
        let mut btree_set: Box<dyn DurableOrderedSet<IntRecord, u32>> =
            DurableBTreeSet::new_boxed();
        btree_set
            .add(int_rec(3, "hello", b"hello_bytes"))
            .expect("unable to add value");
        btree_set
            .add(int_rec(1, "bon_jour", b"bon_jour_bytes"))
            .expect("unable to add value");
        btree_set
            .add(int_rec(2, "guten_tag", b"guten_tag_bytes"))
            .expect("unable to add value");

        assert_eq!(
            Some(int_rec(3, "hello", b"hello_bytes")),
            btree_set.get_by_index(&3).expect("unable to get value")
        );

        assert_eq!(
            vec![
                int_rec(1, "bon_jour", b"bon_jour_bytes"),
                int_rec(2, "guten_tag", b"guten_tag_bytes"),
                int_rec(3, "hello", b"hello_bytes"),
            ],
            btree_set
                .iter()
                .expect("could not iterate")
                .collect::<Vec<_>>()
        )
    }

    /// Using DurableBTreeSet, insert three items, sorted and indexed by integer IDs.
    /// - Verify that the items can be selected based on ranges using the index, and are returned
    ///   in order.
    #[test]
    fn test_btree_range_int() {
        let mut btree_set: Box<dyn DurableOrderedSet<IntRecord, u32>> =
            DurableBTreeSet::new_boxed();
        btree_set
            .add(int_rec(3, "hello", b"hello_bytes"))
            .expect("unable to add value");
        btree_set
            .add(int_rec(1, "bon_jour", b"bon_jour_bytes"))
            .expect("unable to add value");
        btree_set
            .add(int_rec(2, "guten_tag", b"guten_tag_bytes"))
            .expect("unable to add value");

        let total = btree_set.len().expect("Unable to get length");
        assert_eq!(3, total);
        assert_eq!(
            vec![
                int_rec(2, "guten_tag", b"guten_tag_bytes"),
                int_rec(3, "hello", b"hello_bytes"),
            ],
            btree_set
                .range_iter((&2..).into())
                .expect("could not iterate")
                .collect::<Vec<_>>()
        )
    }

    /// Using DurableBTreeSet, insert three items, sorted and indexed by String.
    /// - Verify that the items can be selected based on ranges using the index, and are returned
    ///   in order.
    #[test]
    fn test_btree_range_str() {
        let mut btree_set: Box<dyn DurableOrderedSet<StrRecord, String>> =
            DurableBTreeSet::new_boxed();
        btree_set
            .add(str_rec("hello", b"hello_bytes"))
            .expect("unable to add value");
        btree_set
            .add(str_rec("bon_jour", b"bon_jour_bytes"))
            .expect("unable to add value");
        btree_set
            .add(str_rec("guten_tag", b"guten_tag_bytes"))
            .expect("unable to add value");

        let total = btree_set.len().expect("Unable to get length");
        assert_eq!(3, total);
        assert_eq!(
            vec![
                str_rec("guten_tag", b"guten_tag_bytes"),
                str_rec("hello", b"hello_bytes"),
            ],
            btree_set
                .range_iter((&"g".to_string()..).into())
                .expect("could not iterate")
                .collect::<Vec<_>>()
        )
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    struct IntRecord(u32, String, Vec<u8>);

    fn int_rec(id: u32, name: &str, value: &[u8]) -> IntRecord {
        IntRecord(id, name.into(), value.to_vec())
    }

    impl Ord for IntRecord {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }
    }

    impl std::cmp::PartialOrd for IntRecord {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Borrow<u32> for IntRecord {
        fn borrow(&self) -> &u32 {
            &self.0
        }
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    struct StrRecord(String, Vec<u8>);

    fn str_rec(name: &str, value: &[u8]) -> StrRecord {
        StrRecord(name.into(), value.to_vec())
    }

    impl Ord for StrRecord {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }
    }

    impl std::cmp::PartialOrd for StrRecord {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Borrow<String> for StrRecord {
        fn borrow(&self) -> &String {
            &self.0
        }
    }
}
