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

mod gameroom;
mod gameroom_user;
mod notification;
mod xo_games;

pub use gameroom::{
    fetch_gameroom, fetch_gameroom_by_alias, fetch_gameroom_members_by_circuit_id_and_status,
    fetch_gameroom_proposal_with_status, fetch_proposal_by_id, get_gameroom_count,
    get_proposal_count, insert_gameroom, insert_gameroom_members, insert_gameroom_proposal,
    insert_gameroom_services, insert_proposal_vote_record, list_gameroom_members_with_status,
    list_gamerooms_with_paging, list_gamerooms_with_paging_and_status, list_proposals_with_paging,
    update_gameroom_member_status, update_gameroom_proposal_status, update_gameroom_service_status,
    update_gameroom_status,
};
pub use gameroom_user::{fetch_user_by_email, insert_user};
pub use notification::{
    create_new_notification, fetch_notification, fetch_notifications_by_time,
    get_unread_notification_count, insert_gameroom_notification,
    list_unread_notifications_with_paging, update_gameroom_notification,
};
pub use xo_games::{
    fetch_xo_game, get_xo_game_count, insert_xo_game, list_xo_games, update_xo_game,
};
