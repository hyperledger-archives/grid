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

DROP TABLE IF EXISTS grid_circuit;
DROP TABLE IF EXISTS grid_circuit_proposal;
DROP TABLE IF EXISTS grid_circuit_member;
DROP TABLE IF EXISTS grid_circuit_proposal_vote_record;

ALTER TABLE commit RENAME TO block;

ALTER TABLE block RENAME COLUMN commit_id TO block_id;
ALTER TABLE block RENAME COLUMN commit_num TO block_num;
ALTER TABLE chain_record RENAME COLUMN start_commit_num TO start_block_num;
ALTER TABLE chain_record RENAME COLUMN end_commit_num TO end_block_num;
ALTER TABLE reported_value_reporter_to_agent_metadata RENAME COLUMN reported_value_end_block_num TO reported_value_end_block_num;
ALTER TABLE reported_value_reporter_to_agent_metadata RENAME COLUMN reporter_end_block_num TO reporter_end_block_num;
ALTER TABLE reporter_to_agent_metadata RENAME COLUMN reporter_end_block_num TO reporter_end_block_num;

ALTER TABLE block ADD COLUMN state_root_hash;
