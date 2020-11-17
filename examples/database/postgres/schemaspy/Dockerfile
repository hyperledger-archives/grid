# Copyright 2020 Cargill Incorporated
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
FROM schemaspy/schemaspy:snapshot as SCHEMASPY

FROM ubuntu:bionic as TARGET

RUN apt-get update \
    && apt-get install -y -q \
        build-essential \
        openjdk-8-jre \
        libpq-dev \
        postgresql-client \
        graphviz \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir /output && \
    mkdir -p /org/schemaspy/drivers && \
    mkdir /schemaspy

COPY schemaspy/generate .
COPY schemaspy/pgsql11.properties /schemaspy

RUN chmod a+x generate

FROM TARGET

COPY --from=schemaspy /usr/local/lib/schemaspy/schemaspy-6.1.1-SNAPSHOT.jar /schemaspy/schemaSpy.jar

COPY --from=schemaspy /drivers_inc/ /org/schemaspy/drivers

RUN chmod a+x schemaspy/schemaSpy.jar

ENTRYPOINT ["./generate"]
