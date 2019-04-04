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

CREATE TABLE IF NOT EXISTS block (
    block_id VARCHAR(128) CONSTRAINT pk_block_id PRIMARY KEY,
    block_num BIGINT NOT NULL,
    state_root_hash VARCHAR(64) NOT NULL
);

CREATE INDEX IF NOT EXISTS block_num_idx
    ON block (block_num);
