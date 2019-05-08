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
-- ------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS reporter (
    id BIGSERIAL PRIMARY KEY,
    property_name TEXT NOT NULL,
    public_key TEXT NOT NULL,
    authorized BOOLEAN NOT NULL,
    reporter_index INTEGER
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS reporter_idx
    ON reporter (reporter_index);

CREATE TABLE IF NOT EXISTS property (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    property_definition TEXT NOT NULL,
    current_page INTEGER NOT NULL,
    wrapped BOOLEAN NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS property_idx
    ON property (name, end_block_num);

CREATE TABLE IF NOT EXISTS reported_value (
    id BIGSERIAL PRIMARY KEY,
    property_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    reporter_index INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    value_name TEXT NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS reported_value_idx
    ON reported_value (property_name, record_id, end_block_num);

CREATE TABLE IF NOT EXISTS proposal (
    id BIGSERIAL PRIMARY KEY,
    record_id TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    issuing_agent TEXT NOT NULL,
    receiving_agent TEXT NOT NULL,
    role TEXT NOT NULL,
    properties TEXT [] NOT NULL,
    status TEXT NOT NULL,
    terms TEXT NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS proposal_idx
    ON proposal(record_id, end_block_num);

CREATE TABLE IF NOT EXISTS associated_agent (
    id BIGSERIAL PRIMARY KEY,
    record_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    timestamp BIGINT NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS associated_agent_idx
    ON associated_agent(record_id, agent_id);

CREATE TABLE IF NOT EXISTS record (
    id BIGSERIAL PRIMARY KEY,
    record_id TEXT NOT NULL,
    schema TEXT NOT NULL,
    final BOOL NOT NULL,
    owners TEXT [] NOT NULL,
    custodians TEXT [] NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS record_idx
    ON record(record_id); 
