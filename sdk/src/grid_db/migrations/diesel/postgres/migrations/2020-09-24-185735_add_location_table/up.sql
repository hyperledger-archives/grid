-- Copyright 2020 Cargill Incorporated
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

CREATE TABLE IF NOT EXISTS location (
    id BIGSERIAL PRIMARY KEY,
    location_id VARCHAR(256) NOT NULL,
    location_address VARCHAR(70) NOT NULL,
    location_namespace TEXT NOT NULL,
    owner VARCHAR(256) NOT NULL,
    service_id TEXT
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS location_idx
    ON location (location_id, end_commit_num);
