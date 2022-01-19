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

CREATE TABLE purchase_order_temp(
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    workflow_state TEXT NOT NULL,
    buyer_org_id VARCHAR(256) NOT NULL,
    seller_org_id VARCHAR(256) NOT NULL,
    is_closed BOOLEAN NOT NULL,
    accepted_version_id TEXT,
    created_at BIGINT NOT NULL,
    workflow_id TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO purchase_order_temp(
    id,
    purchase_order_uid,
    workflow_state,
    is_closed,
    accepted_version_id,
    start_commit_num,
    end_commit_num,
    service_id
) SELECT
id,
uuid,
workflow_status,
is_closed,
accepted_version_id,
start_commit_num,
end_commit_num,
service_id
FROM purchase_order;

DROP TABLE purchase_order;

ALTER TABLE purchase_order_temp RENAME TO purchase_order;

CREATE TABLE purchase_order_version_temp(
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    version_id TEXT NOT NULL,
    is_draft BOOLEAN NOT NULL,
    current_revision_id BIGINT NOT NULL,
    workflow_state TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO purchase_order_version_temp(
    id,
    purchase_order_uid,
    version_id,
    is_draft,
    current_revision_id,
    start_commit_num,
    end_commit_num,
    service_id
) SELECT
id,
purchase_order_uuid,
version_id,
is_draft,
current_revision_id,
start_commit_num,
end_commit_num,
service_id
FROM purchase_order_version;

DROP TABLE purchase_order_version;

ALTER TABLE purchase_order_version_temp RENAME TO purchase_order_version;

CREATE TABLE purchase_order_version_revision_temp(
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    version_id TEXT NOT NULL,
    revision_id BIGINT NOT NULL,
    order_xml_v3_4 TEXT NOT NULL,
    submitter TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO purchase_order_version_revision_temp(
    id,
    version_id,
    revision_id,
    order_xml_v3_4,
    submitter,
    created_at,
    start_commit_num,
    end_commit_num,
    service_id
) SELECT
id,
version_id,
revision_id,
order_xml_v3_4,
submitter,
created_at,
start_commit_num,
end_commit_num,
service_id
FROM purchase_order_version_revision;

DROP TABLE purchase_order_version_revision;

ALTER TABLE purchase_order_version_revision_temp RENAME TO purchase_order_version_revision;

CREATE TABLE purchase_order_alternate_id_temp(
    id INTEGER PRIMARY KEY,
    purchase_order_uid TEXT NOT NULL,
    alternate_id_type TEXT NOT NULL,
    alternate_id TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO purchase_order_alternate_id_temp(
    id,
    purchase_order_uid,
    alternate_id_type,
    alternate_id,
    start_commit_num,
    end_commit_num,
    service_id
) SELECT
id,
purchase_order_uuid,
alternate_id_type,
alternate_id,
start_commit_num,
end_commit_num,
service_id
FROM purchase_order_alternate_id;

DROP TABLE purchase_order_alternate_id;

ALTER TABLE purchase_order_alternate_id_temp RENAME TO purchase_order_alternate_id;

-- CREATE TABLE purchase_order (
--     id INTEGER PRIMARY KEY,
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
--     id INTEGER PRIMARY KEY,
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
--     id INTEGER PRIMARY KEY,
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
--     id INTEGER PRIMARY KEY,
--     purchase_order_uid TEXT NOT NULL,
--     alternate_id_type TEXT NOT NULL,
--     alternate_id TEXT NOT NULL,
--     start_commit_num BIGINT NOT NULL,
--     end_commit_num BIGINT NOT NULL,
--     service_id TEXT
-- );
