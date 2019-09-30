-- Copyright (c) 2019 Target Brands, Inc.
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

CREATE TABLE IF NOT EXISTS product (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(256) NOT NULL,
    product_address VARCHAR(70) NOT NULL,
    product_namespace TEXT NOT NULL,
    owner VARCHAR(256) NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS product_idx
    ON product (product_id, end_block_num);
