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

CREATE TABLE IF NOT EXISTS chain_record (
    id BIGSERIAL PRIMARY KEY,
    start_block_num BIGINT NOT NULL,
    end_block_num BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent (
    id BIGSERIAL CONSTRAINT pk_agent PRIMARY KEY,
    public_key VARCHAR(70) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    active BOOLEAN NOT NULL,
    roles TEXT [] NOT NULL,
    metadata JSON NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS agent_pub_key_block_num_idx
    ON agent (public_key, end_block_num);

CREATE TABLE IF NOT EXISTS organization (
    id BIGSERIAL CONSTRAINT pk_organization PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    name VARCHAR(256) NOT NULL,
    address VARCHAR(256) NOT NULL,
    metadata JSON [] NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS org_id_block_num_idx
    ON organization (org_id, end_block_num);
