-- Copyright 2022 Cargill Incorporated
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

ALTER TABLE purchase_order
RENAME COLUMN uuid TO purchase_order_uid;

ALTER TABLE purchase_order
RENAME COLUMN workflow_status TO workflow_state;

ALTER TABLE purchase_order
ALTER COLUMN accepted_version_id DROP NOT NULL;

ALTER TABLE purchase_order
DROP COLUMN org_id;

ALTER TABLE purchase_order
ADD COLUMN buyer_org_id VARCHAR(256) NOT NULL,
ADD COLUMN seller_org_id VARCHAR(256) NOT NULL,
ADD COLUMN workflow_id VARCHAR(256) NOT NULL,
ADD COLUMN created_at BIGINT NOT NULL;

ALTER TABLE purchase_order_version
RENAME COLUMN purchase_order_uuid TO purchase_order_uid;

ALTER TABLE purchase_order_version
DROP COLUMN org_id;

ALTER TABLE purchase_order_version
ADD COLUMN workflow_state TEXT NOT NULL;

ALTER TABLE purchase_order_version
ALTER COLUMN current_revision_id TYPE BIGINT USING current_revision_id::BIGINT;

ALTER TABLE purchase_order_version_revision
DROP COLUMN org_id;

ALTER TABLE purchase_order_version_revision
ALTER COLUMN revision_id TYPE BIGINT USING revision_id::BIGINT;

ALTER TABLE purchase_order_version_revision
ADD COLUMN purchase_order_uid TEXT NOT NULL;

ALTER TABLE purchase_order_alternate_id
RENAME COLUMN purchase_order_uuid TO purchase_order_uid;

ALTER TABLE purchase_order_alternate_id
DROP COLUMN org_id;

-- CREATE TABLE purchase_order (
--     id BIGSERIAL PRIMARY KEY,
--     purchase_order_uid TEXT NOT NULL,
--     workflow_state TEXT NOT NULL,
--     buyer_org_id VARCHAR(256) NOT NULL,
--     seller_org_id VARCHAR(256) NOT NULL,
--     is_closed BOOLEAN NOT NULL,
--     accepted_version_id TEXT,
--     created_at BIGINT NOT NULL,
--     workflow_id TEXT NOT NULL,
--     start_commit_num BIGINT NOT NULL,
--     end_commit_num BIGINT NOT NULL,
--     service_id TEXT
-- );

-- CREATE TABLE purchase_order_version (
--     id BIGSERIAL PRIMARY KEY,
--     purchase_order_uid TEXT NOT NULL,
--     version_id TEXT NOT NULL,
--     is_draft BOOLEAN NOT NULL,
--     current_revision_id BIGINT NOT NULL,
--     workflow_state TEXT NOT NULL,
--     start_commit_num BIGINT NOT NULL,
--     end_commit_num BIGINT NOT NULL,
--     service_id TEXT
-- );

-- CREATE TABLE purchase_order_version_revision (
--     id BIGSERIAL PRIMARY KEY,
--     purchase_order_uid TEXT NOT NULL,
--     version_id TEXT NOT NULL,
--     revision_id BIGINT NOT NULL,
--     order_xml_v3_4 TEXT NOT NULL,
--     submitter TEXT NOT NULL,
--     created_at BIGINT NOT NULL,
--     start_commit_num BIGINT NOT NULL,
--     end_commit_num BIGINT NOT NULL,
--     service_id TEXT
-- );

-- CREATE TABLE purchase_order_alternate_id (
--     id BIGSERIAL PRIMARY KEY,
--     purchase_order_uid TEXT NOT NULL,
--     alternate_id_type TEXT NOT NULL,
--     alternate_id TEXT NOT NULL,
--     start_commit_num BIGINT NOT NULL,
--     end_commit_num BIGINT NOT NULL,
--     service_id TEXT
-- );
