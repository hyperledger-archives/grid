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

DROP TABLE batches;
DROP TABLE transactions;
DROP TABLE transaction_receipts;

CREATE TABLE batches
  (
     service_id        VARCHAR(17) NOT NULL,
     batch_id          VARCHAR(128) NOT NULL,
     data_change_id    VARCHAR(256),
     signer_public_key VARCHAR(70) NOT NULL,
     trace             BOOLEAN NOT NULL,
     serialized_batch  BLOB NOT NULL,
     submitted         BOOLEAN NOT NULL,
     created_at        INTEGER NOT NULL DEFAULT (cast(strftime('%s') as int)),
     PRIMARY KEY (service_id, batch_id)
  );

CREATE TABLE transactions
  (
     service_id        VARCHAR(17) NOT NULL,
     transaction_id    VARCHAR(128) NOT NULL,
     batch_id          VARCHAR(128) NOT NULL,
     batch_service_id  VARCHAR(17) NOT NULL,
     payload           BLOB NOT NULL,
     family_name       VARCHAR(128) NOT NULL,
     family_version    VARCHAR(16) NOT NULL,
     signer_public_key VARCHAR(70) NOT NULL,
     FOREIGN KEY (batch_service_id, batch_id) REFERENCES batches(service_id, batch_id) ON DELETE CASCADE,
     PRIMARY KEY (service_id, transaction_id)
  );

CREATE TABLE transaction_receipts
  (
     service_id             VARCHAR(17) NOT NULL,
     transaction_id         VARCHAR(70) NOT NULL,
     result_valid           BOOLEAN NOT NULL,
     error_message          TEXT,
     error_data             BLOB,
     serialized_receipt     BLOB NOT NULL,
     external_status        VARCHAR(16),
     external_error_message TEXT,
     PRIMARY KEY (service_id, transaction_id)
  );

CREATE TABLE submissions
  (
     service_id            VARCHAR(17) NOT NULL,
     batch_id              VARCHAR(128) NOT NULL,
     batch_service_id      VARCHAR(17) NOT NULL,
     last_checked          INTEGER,
     times_checked         VARCHAR(32),
     error_type            VARCHAR(64),
     error_message         TEXT,
     created_at            INTEGER NOT NULL DEFAULT (cast(strftime('%s') as int)),
     updated_at            INTEGER NOT NULL DEFAULT (cast(strftime('%s') as int)),
     FOREIGN KEY (batch_service_id, batch_id) REFERENCES batches(service_id, batch_id) ON DELETE CASCADE,
     PRIMARY KEY (service_id, batch_id)
  );

CREATE TABLE batch_statuses
  (
     service_id        VARCHAR(17) NOT NULL,
     batch_id          VARCHAR(128) NOT NULL,
     batch_service_id  VARCHAR(17) NOT NULL,
     dlt_status        VARCHAR(16) NOT NULL,
     created_at        INTEGER NOT NULL DEFAULT (cast(strftime('%s') as int)),
     updated_at        INTEGER NOT NULL DEFAULT (cast(strftime('%s') as int)),
     FOREIGN KEY (batch_service_id, batch_id) REFERENCES batches(service_id, batch_id) ON DELETE CASCADE,
     PRIMARY KEY (service_id, batch_id)
  );

CREATE TRIGGER IF NOT EXISTS set_batch_statuses_updated_at_timestamp
BEFORE UPDATE ON batch_statuses
FOR EACH ROW
BEGIN
    UPDATE batch_statuses
    SET updated_at = (cast(strftime('%s') as int))
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER IF NOT EXISTS set_submissions_updated_at_timestamp
BEFORE UPDATE ON submissions
FOR EACH ROW
BEGIN
    UPDATE submissions
    SET updated_at = (cast(strftime('%s') as int))
    WHERE rowid = NEW.rowid;
END;
