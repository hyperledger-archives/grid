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

ALTER TABLE agent ADD COLUMN roles BYTEA NOT NULL;

ALTER TABLE role DROP COLUMN public_key;
ALTER TABLE role DROP COLUMN role_name;
ALTER TABLE role ADD COLUMN org_id VARCHAR(256) NOT NULL;
ALTER TABLE role ADD COLUMN name VARCHAR(256) NOT NULL;
ALTER TABLE role ADD COLUMN description VARCHAR(256) NOT NULL;
ALTER TABLE role ADD COLUMN permissions BYTEA NOT NULL;
ALTER TABLE role ADD COLUMN allowed_orgs BYTEA NOT NULL;
ALTER TABLE role ADD COLUMN inherit_from BYTEA NOT NULL;

ALTER TABLE organization DROP COLUMN address;
ALTER TABLE organization ADD COLUMN locations BYTEA NOT NULL;

CREATE TABLE alternate_identifier (
    id BIGSERIAL PRIMARY KEY,
    alternate_id VARCHAR(256) NOT NULL,
    id_type VARCHAR(256) NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    service_id TEXT
) INHERITS (chain_record);
