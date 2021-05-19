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

DROP TRIGGER set_pike_agent_timestamp;
DROP TRIGGER set_pike_organization_timestamp;
DROP TRIGGER set_pike_role_timestamp;
DROP TRIGGER set_grid_schema_timestamp;
DROP TRIGGER set_product_timestamp;
DROP TRIGGER set_location_timestamp;

ALTER TABLE pike_agent
DROP COLUMN last_updated;

ALTER TABLE pike_organization
DROP COLUMN last_updated;

ALTER TABLE pike_role
DROP COLUMN last_updated;

ALTER TABLE grid_schema
DROP COLUMN last_updated;

ALTER TABLE product
DROP COLUMN last_updated;

ALTER TABLE location
DROP COLUMN last_updated;
