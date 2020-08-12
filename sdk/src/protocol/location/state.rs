// Copyright 2020 Cargill Incorporated
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

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos;
use crate::protos::schema_state;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use crate::protocol::schema::state::PropertyValue;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocationNamespace {
    GS1,
}

impl Default for LocationNamespace {
    fn default() -> Self {
        LocationNamespace::GS1
    }
}

impl FromProto<protos::location_state::Location_LocationNamespace> for LocationNamespace {
    fn from_proto(
        namespace: protos::location_state::Location_LocationNamespace,
    ) -> Result<Self, ProtoConversionError> {
        match namespace {
            protos::location_state::Location_LocationNamespace::GS1 => Ok(LocationNamespace::GS1),
            protos::location_state::Location_LocationNamespace::UNSET_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert Location_LocationType with type UNSET_TYPE".to_string(),
                ))
            }
        }
    }
}

impl FromNative<LocationNamespace> for protos::location_state::Location_LocationNamespace {
    fn from_native(namespace: LocationNamespace) -> Result<Self, ProtoConversionError> {
        match namespace {
            LocationNamespace::GS1 => Ok(protos::location_state::Location_LocationNamespace::GS1),
        }
    }
}

impl IntoProto<protos::location_state::Location_LocationNamespace> for LocationNamespace {}
impl IntoNative<LocationNamespace> for protos::location_state::Location_LocationNamespace {}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    location_id: String,
    namespace: LocationNamespace,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl Location {
    pub fn location_id(&self) -> &str {
        &self.location_id
    }

    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }

    pub fn into_builder(self) -> LocationBuilder {
        LocationBuilder::new()
            .with_location_id(self.location_id)
            .with_namespace(self.namespace)
            .with_owner(self.owner)
            .with_properties(self.properties)
    }
}

impl FromProto<protos::location_state::Location> for Location {
    fn from_proto(
        location: protos::location_state::Location,
    ) -> Result<Self, ProtoConversionError> {
        Ok(Location {
            location_id: location.get_location_id().to_string(),
            namespace: LocationNamespace::from_proto(location.get_namespace())?,
            owner: location.get_owner().to_string(),
            properties: location
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Location> for protos::location_state::Location {
    fn from_native(location: Location) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::location_state::Location::new();
        proto.set_location_id(location.location_id().to_string());
        proto.set_namespace(location.namespace().into_proto()?);
        proto.set_owner(location.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            location
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<schema_state::PropertyValue>, ProtoConversionError>>()?,
        ));
        Ok(proto)
    }
}

impl IntoBytes for Location {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from Location".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<protos::location_state::Location> for Location {}
impl IntoNative<Location> for protos::location_state::Location {}

#[derive(Debug)]
pub enum LocationBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for LocationBuildError {
    fn description(&self) -> &str {
        match *self {
            LocationBuildError::MissingField(ref msg) => msg,
            LocationBuildError::EmptyVec(ref msg) => msg,
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            LocationBuildError::MissingField(_) => None,
            LocationBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for LocationBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LocationBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
            LocationBuildError::EmptyVec(ref s) => write!(f, "\"{}\" must not be empty", s),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct LocationBuilder {
    pub location_id: Option<String>,
    pub namespace: Option<LocationNamespace>,
    pub owner: Option<String>,
    pub properties: Option<Vec<PropertyValue>>,
}

impl LocationBuilder {
    pub fn new() -> Self {
        LocationBuilder::default()
    }

    pub fn with_location_id(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn with_namespace(mut self, namespace: LocationNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn build(self) -> Result<Location, LocationBuildError> {
        let location_id = self.location_id.ok_or_else(|| {
            LocationBuildError::MissingField("'location_id' field is required".to_string())
        })?;

        let namespace = self.namespace.ok_or_else(|| {
            LocationBuildError::MissingField("'namespace' field is required".to_string())
        })?;

        let owner = self.owner.ok_or_else(|| {
            LocationBuildError::MissingField("'owner' field is required".to_string())
        })?;

        let properties = self.properties.ok_or_else(|| {
            LocationBuildError::MissingField("'properties' field is required".to_string())
        })?;

        Ok(Location {
            location_id,
            namespace,
            owner,
            properties,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationList {
    locations: Vec<Location>,
}

impl LocationList {
    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn into_builder(self) -> LocationListBuilder {
        LocationListBuilder::new().with_locations(self.locations)
    }
}

impl FromProto<protos::location_state::LocationList> for LocationList {
    fn from_proto(
        location_list: protos::location_state::LocationList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(LocationList {
            locations: location_list
                .get_entries()
                .to_vec()
                .into_iter()
                .map(Location::from_proto)
                .collect::<Result<Vec<Location>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<LocationList> for protos::location_state::LocationList {
    fn from_native(location_list: LocationList) -> Result<Self, ProtoConversionError> {
        let mut location_list_proto = protos::location_state::LocationList::new();

        location_list_proto.set_entries(RepeatedField::from_vec(
            location_list
                .locations()
                .to_vec()
                .into_iter()
                .map(Location::into_proto)
                .collect::<Result<Vec<protos::location_state::Location>, ProtoConversionError>>()?,
        ));

        Ok(location_list_proto)
    }
}

impl FromBytes<LocationList> for LocationList {
    fn from_bytes(bytes: &[u8]) -> Result<LocationList, ProtoConversionError> {
        let proto: protos::location_state::LocationList = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get LocationList from bytes".to_string(),
                )
            })?;

        proto.into_native()
    }
}

impl IntoBytes for LocationList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from LocationList".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<protos::location_state::LocationList> for LocationList {}
impl IntoNative<LocationList> for protos::location_state::LocationList {}

#[derive(Default, Clone)]
pub struct LocationListBuilder {
    pub locations: Option<Vec<Location>>,
}

impl LocationListBuilder {
    pub fn new() -> Self {
        LocationListBuilder::default()
    }

    pub fn with_locations(mut self, locations: Vec<Location>) -> LocationListBuilder {
        self.locations = Some(locations);
        self
    }

    pub fn build(self) -> Result<LocationList, LocationBuildError> {
        // Product values are not required
        let locations = self
            .locations
            .ok_or_else(|| LocationBuildError::MissingField("locations".to_string()))?;

        let locations = {
            if locations.is_empty() {
                return Err(LocationBuildError::EmptyVec("locations".to_string()));
            } else {
                locations
            }
        };

        Ok(LocationList { locations })
    }
}
