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

//! Durable sets, both ordered and unordered. Implementations of these sets must be thread-safe.

pub mod mem;

use std::borrow::Borrow;
use std::cmp::Ord;
use std::error::Error;
use std::fmt;
use std::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A Durable Set.
///
/// This trait provides an API for interacting with Durable sets that may be backed by a
/// wide variety of persistent storage.
pub trait DurableSet: Send + Sync {
    type Item: Send;

    /// Add an item to the set.
    fn add(&mut self, item: Self::Item) -> Result<(), DurableSetError>;

    /// Remove an item to the set.
    fn remove(&mut self, item: &Self::Item) -> Result<Option<Self::Item>, DurableSetError>;

    /// Get a value based on the query item.
    fn contains(&self, item: &Self::Item) -> Result<bool, DurableSetError>;

    /// Returns an iterator over the contents of the set.
    fn iter<'a>(&'a self) -> Result<Box<(dyn Iterator<Item = Self::Item> + 'a)>, DurableSetError>;

    /// Returns the count of the values in the set.
    fn len(&self) -> Result<u64, DurableSetError>;

    fn is_empty(&self) -> Result<bool, DurableSetError> {
        Ok(self.len()? == 0)
    }
}

/// A Durable, Ordered Set.
///
/// This trait extends the API `DurableSet` to include requirements around ordering of value, as
/// well as Indexing on a sub-value.
///
/// This set contains values of type `V`, which may be indexed by `Index` type.  Items in the set
/// can be selected based on their `Index` value.
pub trait DurableOrderedSet<V, Index>: DurableSet<Item = V>
where
    Index: Ord + Send,
    V: Send + Ord + Borrow<Index>,
{
    /// Get a value based on the index.
    fn get_by_index(&self, index_value: &Index) -> Result<Option<Self::Item>, DurableSetError>;

    /// Check for the existence of a value based on the index.
    fn contains_by_index(&self, index_value: &Index) -> Result<bool, DurableSetError>;

    /// Returns an iterator over a range of index keys.
    fn range_iter<'a>(
        &'a self,
        range: DurableRange<&Index>,
    ) -> Result<Box<(dyn Iterator<Item = Self::Item> + 'a)>, DurableSetError>;

    /// Returns the first item in the set, based on the natural Item order.
    fn first(&self) -> Result<Option<Self::Item>, DurableSetError>;

    /// Returns the last item in the set, based on the natural Item order.
    fn last(&self) -> Result<Option<Self::Item>, DurableSetError>;

    /// Clones this instance into a boxed durable ordered set.
    fn clone_boxed_ordered_set(&self) -> Box<dyn DurableOrderedSet<V, Index>>;
}

impl<V, Index> Clone for Box<dyn DurableOrderedSet<V, Index>>
where
    Index: Ord + Send,
    V: Send + Ord + Borrow<Index>,
{
    fn clone(&self) -> Self {
        self.clone_boxed_ordered_set()
    }
}

/// A Range describing the start and end bounds for a range iterator on a DurableOrderedSet.
///
/// This struct is similar to the various implementations of the RangeBounds trait in the standard
/// library, but is necessary for implementing the most generic set of bounds while still allowing
/// DurableOrderedSet to be used as in a boxed-dyn context.
pub struct DurableRange<T> {
    pub start: Bound<T>,
    pub end: Bound<T>,
}

impl<T> From<Range<T>> for DurableRange<T> {
    fn from(range: Range<T>) -> Self {
        Self {
            start: Bound::Included(range.start),
            end: Bound::Excluded(range.end),
        }
    }
}

impl<T> From<RangeInclusive<T>> for DurableRange<T> {
    fn from(range: RangeInclusive<T>) -> Self {
        let (start, end) = range.into_inner();
        Self {
            start: Bound::Included(start),
            end: Bound::Included(end),
        }
    }
}

impl<T> From<RangeFull> for DurableRange<T> {
    fn from(_: RangeFull) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeFrom<T>> for DurableRange<T> {
    fn from(range: RangeFrom<T>) -> Self {
        Self {
            start: Bound::Included(range.start),
            end: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeTo<T>> for DurableRange<T> {
    fn from(range: RangeTo<T>) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Excluded(range.end),
        }
    }
}

impl<T> From<RangeToInclusive<T>> for DurableRange<T> {
    fn from(range: RangeToInclusive<T>) -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Included(range.end),
        }
    }
}

/// An error that may occur with the underlying implementation of the DurableSet
#[derive(Debug)]
pub struct DurableSetError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl DurableSetError {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.into(),
            source: None,
        }
    }

    pub fn with_source(context: &str, source: Box<dyn Error + Send>) -> Self {
        Self {
            context: context.into(),
            source: Some(source),
        }
    }
}

impl Error for DurableSetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for DurableSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}
