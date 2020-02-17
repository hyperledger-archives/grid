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
# ------------------------------------------------------------------------------

FROM ubuntu:bionic as swagger_downloader

RUN apt-get update \
  && apt-get install -y -q \
       curl \
  && apt-get clean \
  && rm -r /var/lib/apt/lists/*

RUN curl \
      -s https://codeload.github.com/swagger-api/swagger-ui/tar.gz/v3.6.0 \
      -o swagger-ui.tar.gz
RUN tar xfz swagger-ui.tar.gz

FROM httpd:2.4

COPY --from=swagger_downloader /swagger-ui-3.6.0/dist/* /usr/local/apache2/htdocs/

RUN sed -ibak \
      's#http://petstore.swagger.io/v2/swagger.json#http://localhost:9000/api/openapi.yml#' \
      /usr/local/apache2/htdocs/index.html

RUN echo "\
\n\
ServerName swagger_ui\n\
AddDefaultCharset utf-8\n\
LoadModule proxy_module modules/mod_proxy.so\n\
LoadModule proxy_http_module modules/mod_proxy_http.so\n\
ProxyPass /api http://splinterd-node-0:8080\n\
ProxyPassReverse /api http://splinterd-node-0:8080\n\
\n\
" >>/usr/local/apache2/conf/httpd.conf

EXPOSE 80

CMD ["apachectl", "-D", "FOREGROUND"]
