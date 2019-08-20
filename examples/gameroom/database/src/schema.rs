/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

table! {
    gameroom_user (public_key) {
        public_key -> Text,
        encrypted_private_key -> Text,
        email -> Text,
        hashed_password -> Text,
    }
}

table! {
    circuit_proposal (id) {
        id -> Text,
        proposal_type -> Text,
        circuit_id -> Text,
        circuit_hash -> Text,
        requester -> Text,
        authorization_type -> Text,
        persistence -> Text,
        routes -> Text,
        circuit_management_type -> Text,
        application_metadata -> Bytea,
        status -> Text,
        created_time -> Timestamp,
        updated_time -> Timestamp,
    }
}

table! {
    proposal_vote_record (id) {
        id -> Int8,
        proposal_id -> Text,
        voter_public_key -> Text,
        vote -> Text,
        created_time -> Timestamp,
    }
}

table! {
    proposal_circuit_member (id) {
        id -> Int8,
        proposal_id -> Text,
        node_id -> Text,
        endpoint -> Text,
    }
}

table! {
    proposal_circuit_service (id) {
        id -> Int8,
        proposal_id -> Text,
        service_id -> Text,
        service_type -> Text,
        allowed_nodes -> Array<Text>,
    }
}

table! {
    gameroom_notification (id) {
        id -> Int8,
        notification_type -> Text,
        requester -> Text,
        target -> Text,
        created_time -> Timestamp,
        read -> Bool,
    }
}
