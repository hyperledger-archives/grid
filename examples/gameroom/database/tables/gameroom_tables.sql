-- Copyright 2019 Cargill Incorporated
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

-- Create tables
CREATE TABLE IF NOT EXISTS gameroom_user (
  public_key                TEXT        PRIMARY KEY,
  encrypted_private_key     TEXT        NOT NULL,
  email                     TEXT        NOT NULL,
  hashed_password           TEXT        NOT NULL
);

CREATE TABLE IF NOT EXISTS circuit_proposal (
  id                        TEXT        PRIMARY KEY,
  proposal_type             TEXT        NOT NULL ,
  circuit_id                TEXT        NOT NULL,
  circuit_hash              TEXT        NOT NULL,
  requester                 TEXT        NOT NULL,
  authorization_type        TEXT        NOT NULL,
  persistence               TEXT        NOT NULL,
  routes                    TEXT        NOT NULL,
  circuit_management_type   TEXT        NOT NULL,
  application_metadata      BYTEA       NOT NULL,
  status                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL
);

CREATE TABLE IF NOT EXISTS proposal_vote_record(
  id                        BIGSERIAL   PRIMARY KEY,
  proposal_id               TEXT        NOT NULL,
  voter_public_key          TEXT        NOT NULL,
  vote                      TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES circuit_proposal(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS proposal_circuit_member (
  id                        BIGSERIAL   PRIMARY KEY,
  proposal_id               TEXT        NOT NULL,
  node_id                   TEXT        NOT NULL,
  endpoint                  TEXT        NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES circuit_proposal(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS proposal_circuit_service (
  id                        BIGSERIAL   PRIMARY KEY,
  proposal_id               TEXT        NOT NULL,
  service_id                TEXT        NOT NULL,
  service_type              TEXT        NOT NULL,
  allowed_nodes             TEXT[][]    NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES circuit_proposal(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS gameroom_notification (
  id                        BIGSERIAL   PRIMARY KEY,
  notification_type         TEXT        NOT NULL,
  requester                 TEXT        NOT NULL,
  target                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  read                      BOOLEAN     NOT NULL
);
