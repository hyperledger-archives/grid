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
-- -----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS notifications (
  id                        TEXT        PRIMARY KEY,
  payload_title             TEXT        NOT NULL,
  payload_body              TEXT        NOT NULL,
  created                   TIMESTAMP   NOT NULL,
  recipients                TEXT[]      NOT NULL
);

CREATE TABLE IF NOT EXISTS notification_properties (
  id                        BIGSERIAL   PRIMARY KEY,
  notification_id           TEXT        NOT NULL,
  property                  TEXT        NOT NULL,
  property_value            TEXT        NOT NULL,
  FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_notifications (
  notification_id           TEXT        PRIMARY KEY,
  user_id                   TEXT        NOT NULL,
  unread                    BOOL        NOT NULL,
  FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE
);
