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

DROP TRIGGER set_batch_statuses_updated_at_timestamp;
DROP TRIGGER set_submissions_updated_at_timestamp;

DROP TABLE batches;
DROP TABLE transactions;
DROP TABLE transaction_receipts;
DROP TABLE submissions;
DROP TABLE batch_statuses;

CREATE TABLE batches (
    header_signature TEXT PRIMARY KEY,
    data_change_id TEXT,
    signer_public_key TEXT NOT NULL,
    trace BOOLEAN NOT NULL,
    serialized_batch TEXT NOT NULL,
    submitted BOOLEAN NOT NULL,
    submission_error VARCHAR(16),
    submission_error_message TEXT,
    dlt_status VARCHAR(16),
    claim_expires DATETIME,
    created DATETIME DEFAULT CURRENT_TIMESTAMP,
    service_id TEXT
);

CREATE TABLE transactions (
    header_signature TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL,
    family_name TEXT NOT NULL,
    family_version TEXT NOT NULL,
    signer_public_key TEXT NOT NULL,
    FOREIGN KEY (batch_id) REFERENCES batches(header_signature) ON DELETE CASCADE
);

CREATE TABLE transaction_receipts (
    id INTEGER PRIMARY KEY,
    transaction_id TEXT UNIQUE,
    result_valid BOOLEAN NOT NULL,
    error_message TEXT,
    error_data TEXT,
    serialized_receipt TEXT NOT NULL,
    external_status VARCHAR(16),
    external_error_message TEXT
);
