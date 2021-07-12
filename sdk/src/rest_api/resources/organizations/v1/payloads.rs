// Copyright 2018-2021 Cargill Incorporated
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

use crate::{
    pike::store::{AlternateId, Organization, OrganizationMetadata},
    rest_api::resources::paging::v1::Paging,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AlternateIdSlice {
    pub id_type: String,
    pub id: String,
}

impl From<&AlternateId> for AlternateIdSlice {
    fn from(id: &AlternateId) -> Self {
        Self {
            id_type: id.alternate_id_type().to_string(),
            id: id.alternate_id().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationSlice {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateIdSlice>,
    pub metadata: Vec<OrganizationMetadataSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationMetadataSlice {
    pub key: String,
    pub value: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationListSlice {
    pub data: Vec<OrganizationSlice>,
    pub paging: Paging,
}

impl From<Organization> for OrganizationSlice {
    fn from(organization: Organization) -> Self {
        Self {
            org_id: organization.org_id().to_string(),
            name: organization.name().to_string(),
            locations: organization.locations().iter().map(String::from).collect(),
            alternate_ids: organization
                .alternate_ids()
                .iter()
                .map(AlternateIdSlice::from)
                .collect(),
            metadata: organization
                .metadata()
                .iter()
                .map(OrganizationMetadataSlice::from)
                .collect(),
            service_id: organization.service_id().map(ToOwned::to_owned),
            last_updated: organization.last_updated().map(ToOwned::to_owned),
        }
    }
}

impl From<&OrganizationMetadata> for OrganizationMetadataSlice {
    fn from(metadata: &OrganizationMetadata) -> Self {
        Self {
            key: metadata.key().to_string(),
            value: metadata.value().to_string(),
            service_id: metadata.service_id().map(ToOwned::to_owned),
        }
    }
}
