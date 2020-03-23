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

# build stage
FROM node:lts-alpine as build-stage

RUN apk update && apk add python g++ git make && rm -rf /var/cache/apk/*
WORKDIR /grid_ui
COPY package*.json ./
RUN npm config set unsafe-perm true && npm install
COPY . .
ARG REACT_APP_SPLINTER_URL
ENV REACT_APP_SPLINTER_URL $REACT_APP_SPLINTER_URL
ARG REACT_APP_SAPLING_URL
ENV REACT_APP_SAPLING_URL $REACT_APP_SAPLING_URL
RUN yarn build
WORKDIR /grid_ui/build
ARG REPO_VERSION
RUN tar c -z . -f ../grid_ui_v${REPO_VERSION}.tar.gz

# prod stage
FROM httpd:2.4 as prod-stage

ARG SPLINTER_URL
ARG SAPLING_URL

COPY --from=build-stage /grid_ui/grid_ui_*.tar.gz /tmp
RUN tar -xzvf /tmp/grid_ui_*.tar.gz -C /usr/local/apache2/htdocs/
COPY /configs/apache/httpd.conf /usr/local/apache2/conf/httpd.conf

RUN echo "\
  \n\
  ProxyPass /splinterd $SPLINTER_URL\n\
  ProxyPassReverse /splinterd $SPLINTER_URL\n\
  ProxyPass /sapling-dev-server $SAPLING_URL\n\
  ProxyPassReverse /sapling-dev-server $SAPLING_URL\n\
  \n\
  " >>/usr/local/apache2/conf/httpd.conf

EXPOSE 80/tcp
