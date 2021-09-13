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

ALTER TABLE purchase_order_alternate_id RENAME COLUMN purchase_order_uid TO purchase_order_uuid;

CREATE TABLE po_temp (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    workflow_status TEXT NOT NULL,
    is_closed BOOLEAN NOT NULL,
    accepted_version_id TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO po_temp(id,uuid,workflow_status,is_closed,accepted_version_id,start_commit_num,end_commit_num,service_id)
SELECT id,purchase_order_uid,workflow_status,is_closed,accepted_version_id,start_commit_num,end_commit_num,service_id
FROM purchase_order;

DROP TABLE purchase_order;
ALTER TABLE po_temp RENAME TO purchase_order;

CREATE TABLE version_temp (
    id INTEGER PRIMARY KEY,
    purchase_order_uuid TEXT NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    version_id TEXT NOT NULL,
    is_draft BOOLEAN NOT NULL,
    current_revision_id TEXT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO version_temp(id,purchase_order_uuid,version_id,is_draft,current_revision_id,start_commit_num,end_commit_num,service_id)
SELECT id,purchase_order_uid,version_id,is_draft,current_revision_id,start_commit_num,end_commit_num,service_id
FROM purchase_order_version;

DROP TABLE purchase_order_version;
ALTER TABLE version_temp RENAME TO purchase_order_version;

CREATE TABLE rev_temp(
    id INTEGER PRIMARY KEY,
    version_id TEXT NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    revision_id TEXT NOT NULL,
    order_xml_v3_4 TEXT NOT NULL,
    submitter TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    start_commit_num BIGINT NOT NULL,
    end_commit_num BIGINT NOT NULL,
    service_id TEXT
);

INSERT INTO rev_temp(id,version_id,revision_id,order_xml_v3_4,submitter,created_at,start_commit_num,end_commit_num,service_id)
SELECT id,version_id,revision_id,order_xml_v3_4,submitter,created_at,start_commit_num,end_commit_num,service_id
FROM purchase_order_version_revision;
