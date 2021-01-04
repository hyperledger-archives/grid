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
use std::sync::{Arc, Mutex};

use super::OrganizationStore;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::organizations::store::{error::OrganizationStoreError, Organization};

/// Implementation of OrganizationStore that stores Organizations in memory. Useful for when
/// persistence isn't necessary.
#[derive(Clone, Default)]
pub struct MemoryOrganizationStore {
    inner_organization: Arc<Mutex<HashMap<String, Organization>>>,
}

impl MemoryOrganizationStore {
    pub fn new() -> Self {
        MemoryOrganizationStore {
            inner_organization: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl OrganizationStore for MemoryOrganizationStore {
    fn add_organizations(&self, orgs: Vec<Organization>) -> Result<(), OrganizationStoreError> {
        let mut inner_organization = self.inner_organization.lock().map_err(|_| {
            OrganizationStoreError::InternalError(InternalError::with_message(
                "Cannot access organizations: mutex lock poisoned".to_string(),
            ))
        })?;
        for org in orgs {
            inner_organization.insert(org.org_id.clone(), org);
        }
        Ok(())
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        let inner_organization = self.inner_organization.lock().map_err(|_| {
            OrganizationStoreError::InternalError(InternalError::with_message(
                "Cannot access organizations: mutex lock poisoned".to_string(),
            ))
        })?;
        let filtered_orgs = inner_organization
            .iter()
            .filter(|(_, o)| {
                o.service_id.eq(&service_id.map(String::from))
                    && o.end_commit_num.eq(&MAX_COMMIT_NUM)
            })
            .map(|(_, o)| Organization {
                org_id: o.org_id.clone(),
                name: o.name.clone(),
                address: o.address.clone(),
                metadata: o.metadata.clone(),
                start_commit_num: o.start_commit_num,
                end_commit_num: o.end_commit_num,
                service_id: o.service_id.clone(),
            })
            .collect();
        Ok(filtered_orgs)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        let inner_organization = self.inner_organization.lock().map_err(|_| {
            OrganizationStoreError::InternalError(InternalError::with_message(
                "Cannot access organizations: mutex lock poisoned".to_string(),
            ))
        })?;

        for (_, o) in inner_organization.iter() {
            if o.service_id == service_id.map(String::from)
                && o.org_id == org_id
                && o.end_commit_num == MAX_COMMIT_NUM
            {
                return Ok(Some(o.clone()));
            }
        }

        Err(OrganizationStoreError::NotFoundError(format!(
            "Organization with org_id {} not found.",
            org_id,
        )))
    }
}
