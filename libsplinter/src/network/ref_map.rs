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

use std::collections::HashMap;
use std::{error, fmt};

/// A map that will keep track of the number of times an id has been added, and only remove the
/// id once the reference count is 0.
///
/// The RefMap also keeps track of redirects. This allows ids to be changed while keeping the same
/// reference count.
pub struct RefMap {
    // id to reference count
    references: HashMap<String, u64>,
    redirects: HashMap<String, String>,
}

impl RefMap {
    pub fn new() -> Self {
        RefMap {
            references: HashMap::new(),
            redirects: HashMap::new(),
        }
    }

    pub fn add_ref(&mut self, id: String) -> u64 {
        // check if id is for a current id or a redirect
        let ref_id = {
            if let Some(ref_id) = self.redirects.get(&id) {
                ref_id.clone()
            } else {
                id
            }
        };

        if let Some(ref_count) = self.references.remove(&ref_id) {
            let new_ref_count = ref_count + 1;
            self.references.insert(ref_id, new_ref_count);
            new_ref_count
        } else {
            self.references.insert(ref_id, 1);
            1
        }
    }

    pub fn update_ref(&mut self, old_id: String, new_id: String) -> Result<(), RefUpdateError> {
        if let Some(ref_count) = self.references.remove(&old_id) {
            self.references.insert(new_id.clone(), ref_count);

            // update the old forwards
            for (_, v) in self.redirects.iter_mut().filter(|(_, v)| **v == old_id) {
                *v = new_id.clone()
            }

            self.redirects.insert(old_id, new_id);

            Ok(())
        } else {
            Err(RefUpdateError { id: new_id })
        }
    }

    /// remove_ref, return id if the peer id was removed
    ///
    /// This method will panic if the id does not exist.
    pub fn remove_ref(&mut self, id: &str) -> Option<String> {
        // check if id is for a current id or a redirect
        let ref_id = {
            if !self.references.contains_key(id) {
                // if the the id is for an old reference, find updated id
                if let Some(ref_id) = self.redirects.get(id) {
                    ref_id.to_string()
                } else {
                    // if the id is not in the reference or redirects, the reference does not exist
                    panic!("Trying to remove a reference that does not exist: {}", id)
                }
            } else {
                id.to_string()
            }
        };

        let ref_count = match self.references.remove(&ref_id) {
            Some(ref_count) => ref_count,
            None => panic!("Trying to remove a reference that does not exist: {}", id),
        };

        if ref_count == 1 {
            self.references.remove(&ref_id);
            self.redirects.retain(|_, target_id| target_id != id);
            Some(ref_id)
        } else {
            self.references.insert(ref_id, ref_count - 1);
            None
        }
    }
}

#[derive(Debug)]
pub struct RefUpdateError {
    pub id: String,
}

impl error::Error for RefUpdateError {}

impl fmt::Display for RefUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to update ref id for {}", self.id)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // Test that the reference count is set to 1 if the id is new. If the same id is added, again
    // the reference count is incremented.
    #[test]
    fn test_add_ref() {
        let mut ref_map = RefMap::new();
        let ref_count = ref_map.add_ref("test_id".to_string());
        assert_eq!(ref_count, 1);

        let ref_count = ref_map.add_ref("test_id".to_string());
        assert_eq!(ref_count, 2);

        let ref_count = ref_map.add_ref("test_id_2".to_string());
        assert_eq!(ref_count, 1);
    }

    // Test that when removing a reference, if the ref count is greater than 1, the ref count is
    // is decremented and None is retured to notify that caller that the reference has not be fully
    // removed.
    //
    // Then test that if the ref count is 1, the reference is removed and the id is retured, to
    // tell the caller the reference has been removed.
    #[test]
    fn test_remove_ref() {
        let mut ref_map = RefMap::new();
        let ref_count = ref_map.add_ref("test_id".to_string());
        assert_eq!(ref_count, 1);

        let ref_count = ref_map.add_ref("test_id".to_string());
        assert_eq!(ref_count, 2);

        let id = ref_map.remove_ref("test_id");
        assert_eq!(id, None);

        assert_eq!(ref_map.references.get("test_id").cloned(), Some(1 as u64));

        let id = ref_map.remove_ref("test_id");
        assert_eq!(id, Some("test_id".to_string()));
        assert_eq!(ref_map.references.get("test_id"), None);
    }

    // That that if a remove_ref is removed, when the reference does not exist, a panic occurs
    #[test]
    #[should_panic]
    fn test_remove_ref_panic() {
        let mut ref_map = RefMap::new();
        ref_map.remove_ref("test_id");
    }

    // Test that if a reference is updated, the new reference can be used to increase the ref count.
    // Then verify that both ids can be used to remove the reference, returning the updated id on
    // full removal.
    #[test]
    fn test_update_ref() {
        let mut ref_map = RefMap::new();
        let ref_count = ref_map.add_ref("old_id".to_string());
        assert_eq!(ref_count, 1);

        ref_map
            .update_ref("old_id".to_string(), "new_id".to_string())
            .expect("Unable to update reference");

        let ref_count = ref_map.add_ref("new_id".to_string());
        assert_eq!(ref_count, 2);

        let id = ref_map.remove_ref("old_id");
        assert_eq!(id, None);

        let id = ref_map.remove_ref("new_id");
        assert_eq!(id, Some("new_id".to_string()));
    }

    // Test that if an id is updated and then removed, the old id will still panic because the
    // reference has been removed.
    #[test]
    #[should_panic]
    fn test_update_ref_panic() {
        let mut ref_map = RefMap::new();
        let ref_count = ref_map.add_ref("old_id".to_string());
        assert_eq!(ref_count, 1);

        ref_map
            .update_ref("old_id".to_string(), "new_id".to_string())
            .expect("Unable to update reference");

        let id = ref_map.remove_ref("new_id");
        assert_eq!(id, Some("new_id".to_string()));

        ref_map.remove_ref("old_id");
    }
}
