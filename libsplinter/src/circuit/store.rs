// Copyright 2018-2020 Cargill Incorporated
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

use std::convert::TryInto;

use crate::circuit::Circuit;

/// A filter that matches on aspects of a circuit definition.
///
/// Each variant applies to a different field on the circuit defition.
#[derive(Debug)]
pub enum CircuitFilter {
    /// Matches any circuits that have the given node as a member.
    WithMember(String),
}

impl CircuitFilter {
    /// Returns true if the given circuit matches the filter criteria, false otherwise.
    pub fn matches(&self, circuit: &Circuit) -> bool {
        match self {
            CircuitFilter::WithMember(ref member) if circuit.members().contains(member) => true,
            _ => false,
        }
    }
}

pub trait CircuitStore: Send + Sync + Clone {
    /// Return an iterator over the circuits in this store.
    ///
    /// A circuit filter may optionally provided, to reduces the results.
    fn circuits(&self, filter: Option<CircuitFilter>) -> Result<CircuitIter, CircuitStoreError>;

    fn circuit(&self, circuit_name: &str) -> Result<Option<Circuit>, CircuitStoreError>;
}

/// An iterator over circuits, with a well-known count of values.
pub struct CircuitIter {
    inner: Box<dyn Iterator<Item = Circuit> + Send>,
    total: u64,
}

impl CircuitIter {
    /// Construct the new iterator with the given total and an inner iterator.
    pub(crate) fn new(total: u64, inner: Box<dyn Iterator<Item = Circuit> + Send>) -> Self {
        Self { inner, total }
    }

    /// Returns the total count of items in this iterator, without consuming the iterator.
    pub fn total(&self) -> u64 {
        self.total
    }
}

impl Iterator for CircuitIter {
    type Item = Circuit;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // If the u64 bigger than usize on the given platform, we can set this to usize::MAX
        let size = self.total.try_into().unwrap_or(std::usize::MAX);
        (size, Some(size))
    }
}

#[derive(Debug)]
pub struct CircuitStoreError {
    context: String,
    source: Option<Box<dyn std::error::Error + Send + 'static>>,
}

impl std::error::Error for CircuitStoreError {}

impl CircuitStoreError {
    pub fn new(context: String) -> Self {
        Self {
            context,
            source: None,
        }
    }

    pub fn from_source<T: std::error::Error + Send + 'static>(context: String, source: T) -> Self {
        Self {
            context,
            source: Some(Box::new(source)),
        }
    }

    pub fn context(&self) -> String {
        self.context.clone()
    }
}

impl std::fmt::Display for CircuitStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref source) = self.source {
            write!(
                f,
                "CircuitStoreError: Source: {} Context: {}",
                source, self.context
            )
        } else {
            write!(f, "CircuitStoreError: Context {}", self.context)
        }
    }
}
