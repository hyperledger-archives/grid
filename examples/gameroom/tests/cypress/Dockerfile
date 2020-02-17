# Copyright 2018-2020 Cargill Incorporated
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

FROM cypress/base:10

RUN apt-get update \
 && apt-get install -y \
    postgresql-client \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Optionally pass proxy information to get internet connectivity within npm ci
# postinstall hooks when running behind corporate proxies.
ARG http_proxy
ARG https_proxy
ARG no_proxy

WORKDIR app/

# dependencies will be installed only if the package files change
COPY /cypress/package*.json ./
RUN npm install

COPY /cypress .
# verify that Cypress has been installed correctly.
# running this command separately from "cypress run" will also cache its result
# to avoid verifying again when running the tests
RUN npx cypress verify
