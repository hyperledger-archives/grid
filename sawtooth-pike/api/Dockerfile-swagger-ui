# Copyright 2018 Cargill Incorporated
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

FROM ubuntu:bionic

RUN apt-get update \
  && apt-get install -y -q \
       apache2 \
       curl \
       vim \
  && apt-get clean \
  && rm -r /var/lib/apt/lists/*

WORKDIR /var/www

RUN curl \
      -s https://codeload.github.com/swagger-api/swagger-ui/tar.gz/v3.6.0 \
      -o swagger-ui.tar.gz
RUN tar xfz swagger-ui.tar.gz
RUN mv swagger-ui-3.6.0/dist/* /var/www/html/

RUN sed -ibak \
      's#http://petstore.swagger.io/v2/swagger.json#http://localhost:9001/openapi.yaml#' \
      /var/www/html/index.html

EXPOSE 80

CMD ["apachectl", "-D", "FOREGROUND"]
