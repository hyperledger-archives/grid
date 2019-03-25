/* Copyright 2018 Cargill Incorporated

  Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
CREATE TABLE agents (
    public_key VARCHAR(70) PRIMARY KEY NOT NULL,
    org_id VARCHAR(256) NOT NULL,
    active BOOLEAN NOT NULL,
    roles VARCHAR(256) [] NOT NULL,
    metadata JSON [] NOT NULL
);

CREATE TABLE organizations (
    id VARCHAR(256) PRIMARY KEY NOT NULL,
    name VARCHAR(256) NOT NULL,
    address VARCHAR(256) NOT NULL
);
