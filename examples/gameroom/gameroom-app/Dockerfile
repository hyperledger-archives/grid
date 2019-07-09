# Copyright 2019 Cargill Incorporated
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

WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build

# prod stage
FROM httpd:2.4 as prod-stage

RUN echo "\
\n\
ServerName gameroom-app\n\
AddDefaultCharset utf-8\n\
LoadModule proxy_module modules/mod_proxy.so\n\
LoadModule proxy_http_module modules/mod_proxy_http.so\n\
ProxyPass /api http://gameroom-rest-api:8000\n\
ProxyPassReverse /api http://gameroom-rest-api:8000\n\
\n\
" >>/usr/local/apache2/conf/httpd.conf

COPY --from=build-stage /app/dist/ /usr/local/apache2/htdocs/

EXPOSE 80/tcp
