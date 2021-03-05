-- Copyright 2021 Cargill Incorporated
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

DROP VIEW reported_value_reporter_to_agent_metadata;
DROP VIEW reporter_to_agent_metadata;

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

DROP TABLE pike_agent;
DROP TABLE pike_organization;
DROP TABLE pike_agent_role_assoc;
DROP TABLE pike_role;
DROP TABLE pike_organization_metadata;
DROP TABLE pike_organization_alternate_id;
DROP TABLE pike_organization_location_assoc;
DROP TABLE pike_inherit_from;
DROP TABLE pike_permissions;
DROP TABLE pike_allowed_orgs;
DROP TABLE pike_role_state_address_assoc;
