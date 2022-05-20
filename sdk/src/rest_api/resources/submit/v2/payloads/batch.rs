// Copyright 2022 Cargill Incorporated
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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct TrackingBatchResource {
    pub signed_batch: BatchBytes,
    pub service_id: Option<String>,
    pub data_change_id: Option<String>,
}

/// A serialized `Batch`
pub type BatchBytes = Vec<u8>;

/// Represents a list of batches created from the REST API
#[derive(Default, Serialize, Deserialize)]
pub struct SubmitBatchRequest {
    pub batches: Vec<TrackingBatchResource>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
/// A batch's identifying information
pub struct BatchIdentifier {
    pub dlt_batch_id: String,
    pub data_change_id: Option<String>,
    pub service_id: Option<String>,
}
