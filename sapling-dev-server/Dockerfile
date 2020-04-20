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


FROM node:lts-alpine as build-stage

RUN apk update && apk add git

RUN npm config set unsafe-perm true

COPY . .

# Hack to add register-login and profiles saplings until there's a distribution
# strategy for saplings
RUN git clone https://github.com/hyperledger/grid
RUN cp -r grid/grid-ui/saplings/register-login /saplings
RUN cp -r grid/grid-ui/saplings/profile /saplings

ARG PUBLIC_URL_PARTIAL

ENV PUBLIC_URL $PUBLIC_URL_PARTIAL
RUN cd /saplings/register-login && \
  npm install && \
  npm run deploy

ENV PUBLIC_URL ${PUBLIC_URL_PARTIAL}/profile
RUN cd /saplings/profile && \
  npm install && \
  npm run deploy

ENV PUBLIC_URL ${PUBLIC_URL_PARTIAL}/circuits
RUN cd /saplings/circuits && \
  npm install && \
  npm run deploy

FROM httpd:2.4 as prod-stage

RUN echo "\
  \n\
  Header set Access-Control-Allow-Origin "*"\n\
  \n\
  " >>/usr/local/apache2/conf/httpd.conf

COPY --from=build-stage /sapling-dev-server/ /usr/local/apache2/htdocs/

EXPOSE 80/tcp
