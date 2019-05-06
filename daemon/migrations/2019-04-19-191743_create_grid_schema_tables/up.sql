--- Copyright 2019 Cargill Incorporated
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

CREATE TABLE IF NOT EXISTS grid_schema (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    owner TEXT NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS grid_schema_name_block_num_idx
    ON grid_schema (name, end_block_num);

CREATE TABLE IF NOT EXISTS grid_property_definition (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    schema_name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    required BOOLEAN NOT NULL,
    description TEXT NOT NULL,
    number_exponent BIGINT NOT NULL,
    enum_options TEXT [] NOT NULL,
    struct_properties TEXT [] NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS grid_property_definition_name_block_num_idx
    ON grid_property_definition (name, end_block_num);


-- Create the latlong type if it does not already exists;
DO $$
BEGIN
  CREATE TYPE latlong as (
   latitude BIGINT,
   longitude BIGINT
);
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS grid_property_value (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    bytes_value BYTEA,
    boolean_value BOOLEAN,
    number_value BIGINT,
    string_value TEXT,
    enum_value INTEGER,
    struct_values TEXT [],
    lat_long_value latlong
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS grid_property_value_name_block_num_idx
    ON grid_property_value (name, end_block_num);
