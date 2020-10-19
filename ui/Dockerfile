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

# Canopy build stage
FROM node:lts-alpine as canopy-app-build-stage

RUN apk update && apk add python g++ git make && rm -rf /var/cache/apk/*
WORKDIR /ui
COPY ui/grid-ui/package*.json ./
RUN npm config set unsafe-perm true && npm install
COPY ui/grid-ui .
ENV REACT_APP_SPLINTER_URL "/splinterd"
ENV REACT_APP_SAPLING_URL "/sapling-dev-server"
ENV REACT_APP_GRID_URL "/gridd"
RUN npm run build
WORKDIR /ui/build
ARG REPO_VERSION
RUN tar c -z . -f ../grid_ui_v${REPO_VERSION}.tar.gz

# Log the commit hash
COPY .git/ /tmp/.git/
WORKDIR /tmp
RUN git rev-parse HEAD > /commit-hash

# Sapling build stage
FROM node:lts-alpine as sapling-build-stage

RUN apk update && apk add git

RUN npm config set unsafe-perm true

COPY ui/saplings saplings
COPY ui/sapling-dev-server sapling-dev-server
COPY ui/protos protos

ARG PUBLIC_URL_PARTIAL
ENV PUBLIC_URL $PUBLIC_URL_PARTIAL

ENV PUBLIC_URL ${PUBLIC_URL_PARTIAL}/product
RUN cd /saplings/product && \
  npm install && \
  npm run deploy

RUN git clone https://github.com/Cargill/splinter-ui

RUN cp -r splinter-ui/saplings/register-login /saplings && \
  cd saplings/register-login && \
  npm install && \
  npm run deploy

RUN cp -r splinter-ui/saplings/profile /saplings && \
  cd saplings/profile && \
  npm install && \
  npm run deploy

ENV PUBLIC_URL ${PUBLIC_URL_PARTIAL}/circuits
RUN cp -r splinter-ui/saplings/circuits /saplings && \
  cd saplings/circuits && \
  npm install && \
  npm run deploy

WORKDIR /sapling-dev-server
ARG REPO_VERSION
RUN tar c -z . -f ../grid_saplings_v${REPO_VERSION}.tar.gz

# prod stage
FROM httpd:2.4 as prod-stage

COPY --from=canopy-app-build-stage /ui/grid_ui_v*.tar.gz /tmp
COPY --from=canopy-app-build-stage /commit-hash /commit-hash
RUN tar -xzvf /tmp/grid_ui_*.tar.gz -C /usr/local/apache2/htdocs/

COPY --from=sapling-build-stage /grid_saplings_v*.tar.gz /tmp
RUN mkdir /usr/local/apache2/htdocs/sapling-dev-server
RUN tar -xzvf /tmp/grid_saplings_*.tar.gz \
  -C /usr/local/apache2/htdocs/sapling-dev-server/

COPY ui/configs/apache/httpd.conf /usr/local/apache2/conf/httpd.conf

RUN echo "\
  \n\
  LoadModule headers_module modules/mod_headers.so\n\
  ProxyPass /splinterd \${SPLINTER_URL}\n\
  ProxyPassReverse /splinterd \${SPLINTER_URL}\n\
  <Directory "/usr/local/apache2/htdocs/sapling-dev-server">\n\
  Header set Access-Control-Allow-Origin "*"\n\
  </Directory>\n\
  \n\
  " >>/usr/local/apache2/conf/httpd.conf

EXPOSE 80/tcp
