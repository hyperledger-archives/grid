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

ALTER TABLE pike_agent
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

ALTER TABLE pike_organization
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

ALTER TABLE pike_role
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

ALTER TABLE grid_schema
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

ALTER TABLE product
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

ALTER TABLE location
ADD COLUMN last_updated TIMESTAMP DEFAULT NULL;

CREATE TRIGGER set_pike_agent_timestamp
AFTER INSERT ON pike_agent
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE pike_agent
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER set_pike_organization_timestamp
AFTER INSERT ON pike_organization
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE pike_organization
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER set_pike_role_timestamp
AFTER INSERT ON pike_role
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE pike_role
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER set_grid_schema_timestamp
AFTER INSERT ON grid_schema
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE grid_schema
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER set_product_timestamp
AFTER INSERT ON product
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE product
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER set_location_timestamp
AFTER INSERT ON location
FOR EACH ROW
WHEN NEW.last_updated IS NULL
BEGIN
    UPDATE location
    SET last_updated = CURRENT_TIMESTAMP
    WHERE rowid = NEW.rowid;
END;
