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

mod gameroom_user;
mod proposal;

pub use gameroom_user::{fetch_user_by_email, insert_user};
pub use proposal::{
    fetch_circuit_members_by_proposal_id, fetch_circuit_proposal_with_status, fetch_proposal_by_id,
    get_proposal_count, insert_circuit_member, insert_circuit_proposal, insert_circuit_service,
    insert_proposal_information, insert_proposal_vote_record, list_proposal_circuit_members,
    list_proposals_with_paging, update_circuit_proposal_status,
};
