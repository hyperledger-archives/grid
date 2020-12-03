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

CREATE TABLE commits (
    id INTEGER PRIMARY KEY,
    commit_id VARCHAR(128),
    commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE chain_record (
    id INTEGER PRIMARY KEY,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE grid_circuit (
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

CREATE TABLE grid_circuit_proposal (
    id INTEGER PRIMARY KEY,
    proposal_type TEXT NOT NULL,
    circuit_id TEXT NOT NULL,
    circuit_hash TEXT NOT NULL,
    requester TEXT NOT NULL,
    requester_node_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL
);

CREATE TABLE grid_circuit_member (
    id INTEGER PRIMARY KEY,
    circuit_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    status TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL
);

CREATE TABLE grid_circuit_proposal_vote_record (
    id INTEGER PRIMARY KEY,
    proposal_id BIGSERIAL NOT NULL,
    voter_public_key TEXT NOT NULL,
    voter_node_id TEXT NOT NULL,
    vote TEXT NOT NULL,
    created_time TIMESTAMP NOT NULL
);

CREATE TABLE agent (
    id INTEGER PRIMARY KEY,
    public_key VARCHAR(70) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    active BOOLEAN NOT NULL,
    metadata BYTEA NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE role (
    id INTEGER PRIMARY KEY,
    public_key VARCHAR(70) NOT NULL,
    role_name TEXT NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE organization (
    id INTEGER PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    name VARCHAR(256) NOT NULL,
    address VARCHAR(256) NOT NULL,
    metadata BYTEA NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE associated_agent (
    id INTEGER PRIMARY KEY,
    record_id TEXT NOT NULL,
    role TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE property (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    property_definition TEXT NOT NULL,
    current_page INTEGER NOT NULL,
    wrapped BOOLEAN NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE proposal (
    id INTEGER PRIMARY KEY,
    record_id TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    issuing_agent TEXT NOT NULL,
    receiving_agent TEXT NOT NULL,
    role TEXT NOT NULL,
    properties TEXT NOT NULL,
    status TEXT NOT NULL,
    terms TEXT NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE record (
    id INTEGER PRIMARY KEY,
    record_id TEXT NOT NULL,
    schema TEXT NOT NULL,
    final BOOL NOT NULL,
    owners TEXT NOT NULL,
    custodians TEXT NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE reported_value (
    id INTEGER PRIMARY KEY,
    property_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    reporter_index INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    data_type TEXT NOT NULL,
    bytes_value BYTEA,
    boolean_value BOOLEAN,
    number_value BIGINT,
    string_value TEXT,
    enum_value INTEGER,
    parent_name TEXT,
    latitude_value BIGINT,
    longitude_value BIGINT,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE reporter (
    id INTEGER PRIMARY KEY,
    property_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    authorized BOOLEAN NOT NULL,
    reporter_index INTEGER NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE VIEW reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         public_key,
         authorized,
         reporter_index,
         metadata,
         service_id,
         reporter_end_commit_num
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY agent_end_commit_num) AS RowNum,
                 *
          FROM   (SELECT reporter.id,
                         reporter.property_name,
                         reporter.record_id,
                         reporter.reporter_index,
                         reporter.authorized,
                         reporter.public_key,
                         reporter.end_commit_num AS "reporter_end_commit_num",
                         agent.end_commit_num    AS "agent_end_commit_num",
                         agent.metadata,
                         agent.service_id
                  FROM   reporter
                         LEFT JOIN agent
                                ON reporter.public_key = agent.public_key
                                   AND reporter.end_commit_num <=
                                       agent.end_commit_num) AS
                 join_tables) X
  WHERE  rownum = 1;

CREATE VIEW reported_value_reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         reporter_index,
         timestamp,
         data_type,
         bytes_value,
         boolean_value,
         number_value,
         string_value,
         enum_value,
         parent_name,
         latitude_value,
         longitude_value,
         public_key,
         authorized,
         metadata,
         reported_value_end_commit_num,
         reporter_end_commit_num,
         service_id
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY reporter_end_commit_num) AS RowNum,
                 *
          FROM   (SELECT reported_value.id,
                         reported_value.property_name,
                         reported_value.record_id,
                         reported_value.reporter_index,
                         reported_value.timestamp,
                         reported_value.data_type,
                         reported_value.bytes_value,
                         reported_value.boolean_value,
                         reported_value.number_value,
                         reported_value.string_value,
                         reported_value.enum_value,
                         reported_value.parent_name,
                         reported_value.latitude_value,
                         reported_value.longitude_value,
                         reported_value.end_commit_num AS
                         "reported_value_end_commit_num",
                         reporter_to_agent_metadata.reporter_end_commit_num,
                         reporter_to_agent_metadata.public_key,
                         reporter_to_agent_metadata.authorized,
                         reporter_to_agent_metadata.metadata,
                         reported_value.service_id
                  FROM   reported_value
                         LEFT JOIN reporter_to_agent_metadata
                                ON reported_value.record_id =
                                   reporter_to_agent_metadata.record_id
                                   AND reported_value.property_name =
                                       reporter_to_agent_metadata.property_name
                                   AND reported_value.reporter_index =
                                       reporter_to_agent_metadata.reporter_index
                                   AND reported_value.end_commit_num <=
  reporter_to_agent_metadata.reporter_end_commit_num) AS
  join_tables) X
  WHERE  rownum = 1;

CREATE TABLE grid_schema (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    owner TEXT NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE grid_property_definition (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    schema_name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    required BOOLEAN NOT NULL,
    description TEXT NOT NULL,
    number_exponent BIGINT NOT NULL,
    enum_options TEXT NOT NULL,
    parent_name TEXT,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE product (
    id INTEGER PRIMARY KEY,
    product_id VARCHAR(256) NOT NULL,
    product_address VARCHAR(70) NOT NULL,
    product_namespace TEXT NOT NULL,
    owner VARCHAR(256) NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE product_property_value (
    id INTEGER PRIMARY KEY,
    product_id VARCHAR(256) NOT NULL,
    product_address VARCHAR(70) NOT NULL,
    property_name TEXT NOT NULL,
    parent_property TEXT,
    data_type TEXT NOT NULL,
    bytes_value BYTEA,
    number_value BIGINT,
    boolean_value BOOLEAN,
    string_value TEXT,
    enum_value INTEGER,
    latitude_value BIGINT,
    longitude_value BIGINT,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE location (
    id INTEGER PRIMARY KEY,
    location_id VARCHAR(256) NOT NULL,
    location_address VARCHAR(70) NOT NULL,
    location_namespace TEXT NOT NULL,
    owner VARCHAR(256) NOT NULL,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);

CREATE TABLE location_attribute (
    id INTEGER PRIMARY KEY,
    location_id VARCHAR(256) NOT NULL,
    location_address VARCHAR(70) NOT NULL,
    property_name TEXT NOT NULL,
    parent_property_name TEXT,
    data_type TEXT NOT NULL,
    bytes_value BYTEA,
    boolean_value BOOLEAN,
    number_value BIGINT,
    string_value TEXT,
    enum_value INTEGER,
    latitude_value BIGINT,
    longitude_value BIGINT,
    service_id TEXT,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL
);
