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

FROM hyperledger/grid-dev as gridd-builder

COPY cli /build/cli
COPY daemon /build/daemon
COPY sdk /build/sdk

# Build the gridd package
WORKDIR /build/daemon
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" Cargo.toml
RUN cargo deb --deb-version $REPO_VERSION

# Build the grid-cli package
WORKDIR /build/cli
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" Cargo.toml
RUN cargo deb --deb-version $REPO_VERSION

# -------------=== gridd docker build ===-------------
FROM ubuntu:bionic

COPY --from=gridd-builder /build/target/debian/grid-cli_*.deb /tmp
COPY --from=gridd-builder /build/target/debian/grid-daemon_*.deb /tmp

RUN apt-get update \
 && apt-get install -y -q \
    postgresql-client \
 && dpkg --unpack /tmp/grid*.deb \
 && apt-get -f -y install

CMD ["gridd"]
