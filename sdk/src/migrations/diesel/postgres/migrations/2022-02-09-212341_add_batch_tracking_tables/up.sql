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

DROP TABLE batches CASCADE;
DROP TABLE transactions;
DROP TABLE transaction_receipts;

CREATE TABLE batches
  (
     batch_id          VARCHAR(70) PRIMARY KEY,
     service_id        VARCHAR(17) NOT NULL,
     batch_header      VARCHAR(128) NOT NULL,
     data_change_id    VARCHAR(256),
     signer_public_key VARCHAR(70) NOT NULL,
     trace             BOOLEAN NOT NULL,
     serialized_batch  BYTEA NOT NULL,
     submitted         BOOLEAN NOT NULL,
     created_at        TIMESTAMP NOT NULL
  );

CREATE TABLE transactions
  (
     transaction_id     VARCHAR(70) PRIMARY KEY,
     service_id         VARCHAR(17) NOT NULL,
     transaction_header VARCHAR(128) NOT NULL,
     batch_id           VARCHAR(128) NOT NULL,
     payload            BYTEA NOT NULL,
     family_name        VARCHAR(128) NOT NULL,
     family_version     VARCHAR(16) NOT NULL,
     signer_public_key  VARCHAR(70) NOT NULL,
     FOREIGN KEY (batch_id) REFERENCES batches(batch_id) ON DELETE CASCADE
  );

CREATE TABLE transaction_receipts
  (
     transaction_id         VARCHAR(70) PRIMARY KEY,
     service_id             VARCHAR(17) NOT NULL,
     result_valid           BOOLEAN NOT NULL,
     error_message          TEXT,
     error_data             BYTEA,
     serialized_receipt     BYTEA NOT NULL,
     external_status        VARCHAR(16),
     external_error_message TEXT,
     FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id) ON DELETE CASCADE
  );

CREATE TABLE submissions
  (
     batch_id              VARCHAR(70) PRIMARY KEY,
     service_id            VARCHAR(17) NOT NULL,
     last_checked          TIMESTAMP,
     times_checked         VARCHAR(32),
     error_type            VARCHAR(64),
     error_message         TEXT,
     created_at            TIMESTAMP NOT NULL,
     updated_at            TIMESTAMP NOT NULL,
     FOREIGN KEY (batch_id) REFERENCES batches(batch_id) ON DELETE CASCADE
  );

CREATE TABLE batch_statuses
  (
     batch_id          VARCHAR(70) PRIMARY KEY,
     service_id        VARCHAR(17) NOT NULL,
     dlt_status        VARCHAR(16) NOT NULL,
     created_at        TIMESTAMP NOT NULL,
     updated_at        TIMESTAMP NOT NULL,
     FOREIGN KEY (batch_id) REFERENCES batches(batch_id) ON DELETE CASCADE
  );
