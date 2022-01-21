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
RENAME COLUMN purchase_order_uid TO uuid;

ALTER TABLE purchase_order
RENAME COLUMN workflow_state TO workflow_status;

ALTER TABLE purchase_order
ALTER COLUMN accepted_version_id SET NOT NULL;

ALTER TABLE purchase_order
DROP COLUMN buyer_org_id,
DROP COLUMN seller_org_id,
DROP COLUMN workflow_id,
DROP COLUMN created_at;

ALTER TABLE purchase_order
ADD COLUMN org_id VARCHAR(256) NOT NULL;

ALTER TABLE purchase_order_version
RENAME COLUMN purchase_order_uid TO purchase_order_uuid;

ALTER TABLE purchase_order_version
DROP COLUMN workflow_state;

ALTER TABLE purchase_order_version
ADD COLUMN org_id VARCHAR(256) NOT NULL;

ALTER TABLE purchase_order_version
ALTER COLUMN current_revision_id TYPE TEXT USING current_revision_id::TEXT;

ALTER TABLE purchase_order_version_revision
DROP COLUMN purchase_order_uid;

ALTER TABLE purchase_order_version_revision
ADD COLUMN org_id VARCHAR(256) NOT NULL;

ALTER TABLE purchase_order_version_revision
ALTER COLUMN revision_id TYPE TEXT USING revision_id::TEXT;

ALTER TABLE purchase_order_alternate_id
RENAME COLUMN purchase_order_uid TO purchase_order_uuid;

ALTER TABLE purchase_order_alternate_id
ADD COLUMN org_id VARCHAR(256) NOT NULL;
