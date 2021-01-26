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

ALTER TABLE agent DROP COLUMN roles;

ALTER TABLE role DROP COLUMN org_id;
ALTER TABLE role DROP COLUMN name;
ALTER TABLE role DROP COLUMN description;
ALTER TABLE role DROP COLUMN permissions;
ALTER TABLE role DROP COLUMN allowed_orgs;
ALTER TABLE role DROP COLUMN inherit_from;
ALTER TABLE role ADD COLUMN public_key public_key VARCHAR(70) NOT NULL;
ALTER TABLE role ADD COLUMN role_name TEXT NOT NULL;

ALTER TABLE organization DROP COLUMN locations;
ALTER TABLE organization ADD COLUMN address VARCHAR(256) NOT NULL;

DROP TABLE alternate_identifier;
