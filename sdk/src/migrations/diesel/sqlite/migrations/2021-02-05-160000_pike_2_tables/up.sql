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
DROP TABLE agent;
DROP TABLE role;
DROP TABLE organization;

CREATE TABLE pike_agent (
    id BIGSERIAL PRIMARY KEY,
    public_key VARCHAR(70) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    active BOOLEAN NOT NULL,
    metadata BYTEA NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_organization (
    id BIGSERIAL PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    name VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_agent_role_assoc (
    id BIGSERIAL PRIMARY KEY,
    agent_public_key VARCHAR(70) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    role_name VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_role (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    description TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_organization_metadata (
    id BIGSERIAL PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    key VARCHAR NOT NULL,
    value BYTEA NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_organization_alternate_id (
    id BIGSERIAL PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    alternate_id_type VARCHAR NOT NULL,
    alternate_id VARCHAR NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_organization_location_assoc (
    id BIGSERIAL PRIMARY KEY,
    org_id VARCHAR(256) NOT NULL,
    location_id VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_inherit_from (
    id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(256) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    inherit_from_role_name VARCHAR(256) NOT NULL,
    inherit_from_org_id VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_permissions (
    id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(256) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    name VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

CREATE TABLE pike_allowed_orgs (
    id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(256) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    allowed_org_id VARCHAR(256) NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
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
                         pike_agent.end_commit_num    AS "agent_end_commit_num",
                         pike_agent.metadata,
                         pike_agent.service_id
                  FROM   reporter
                         LEFT JOIN pike_agent
                                ON reporter.public_key = pike_agent.public_key
                                   AND reporter.end_commit_num <=
                                       pike_agent.end_commit_num) AS
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
