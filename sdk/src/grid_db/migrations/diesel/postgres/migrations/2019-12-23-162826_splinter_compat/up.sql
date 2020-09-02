-- Copyright 2019 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
-- -----------------------------------------------------------------------------

ALTER TABLE block DROP COLUMN state_root_hash;

ALTER TABLE block RENAME COLUMN block_id TO commit_id;
ALTER TABLE block RENAME COLUMN block_num TO commit_num;
ALTER TABLE chain_record RENAME COLUMN start_block_num TO start_commit_num;
ALTER TABLE chain_record RENAME COLUMN end_block_num TO end_commit_num;
ALTER TABLE reported_value_reporter_to_agent_metadata RENAME COLUMN reported_value_end_block_num TO reported_value_end_commit_num;
ALTER TABLE reported_value_reporter_to_agent_metadata RENAME COLUMN reporter_end_block_num TO reporter_end_commit_num;
ALTER TABLE reporter_to_agent_metadata RENAME COLUMN reporter_end_block_num TO reporter_end_commit_num;

ALTER TABLE block RENAME TO commit;

CREATE TABLE IF NOT EXISTS grid_circuit (
    circuit_id TEXT PRIMARY KEY,
    authorization_type TEXT NOT NULL,
    persistence TEXT NOT NULL,
    durability TEXT NOT NULL,
    routes TEXT NOT NULL,
    circuit_management_type TEXT NOT NULL,
    alias TEXT NOT NULL,
    status TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS grid_circuit_proposal (
    id BIGSERIAL PRIMARY KEY,
    proposal_type TEXT NOT NULL,
    circuit_id TEXT NOT NULL,
    circuit_hash TEXT NOT NULL,
    requester TEXT NOT NULL,
    requester_node_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS grid_circuit_member (
    id BIGSERIAL PRIMARY KEY,
    circuit_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    status TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS grid_circuit_proposal_vote_record (
    id BIGSERIAL PRIMARY KEY,
    proposal_id BIGSERIAL NOT NULL,
    voter_public_key TEXT NOT NULL,
    voter_node_id TEXT NOT NULL,
    vote TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL
);
