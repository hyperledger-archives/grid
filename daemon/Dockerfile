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

FROM hyperledger/grid-dev:v11 as gridd-builder

ENV GRID_FORCE_PANDOC=true

# This is temporary until hyperledger/grid-dev updates
RUN apt-get update && apt-get install -y -q --no-install-recommends pandoc

# Copy over Cargo.toml files
COPY Cargo.toml /build/Cargo.toml
COPY cli/Cargo.toml /build/cli/Cargo.toml
COPY daemon/Cargo.toml /build/daemon/Cargo.toml
COPY griddle/Cargo.toml /build/griddle/Cargo.toml
COPY sdk/Cargo.toml /build/sdk/Cargo.toml

COPY contracts/location/Cargo.toml /build/contracts/location/Cargo.toml
COPY contracts/pike/Cargo.toml /build/contracts/pike/Cargo.toml
COPY contracts/product/Cargo.toml /build/contracts/product/Cargo.toml
COPY contracts/purchase_order/Cargo.toml /build/contracts/purchase_order/Cargo.toml
COPY contracts/schema/Cargo.toml /build/contracts/schema/Cargo.toml
COPY contracts/track_and_trace/Cargo.toml /build/contracts/track_and_trace/Cargo.toml

# Copy over build files
COPY cli /build/cli
COPY daemon /build/daemon
COPY sdk /build/sdk

# Copy over files needed for examples
COPY sdk/src/data_validation/xml/xsd/product/ /build/daemon/packaging/xsd/product/

# Build the gridd package
ARG CARGO_ARGS
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" daemon/Cargo.toml \
&& cargo build --manifest-path=daemon/Cargo.toml --release $CARGO_ARGS
WORKDIR /build/daemon
RUN cargo deb --no-build --deb-version $REPO_VERSION --manifest-path ./Cargo.toml

WORKDIR /build

# Build the grid-cli package
ARG CARGO_ARGS
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" cli/Cargo.toml \
 && cargo build --manifest-path=cli/Cargo.toml --release $CARGO_ARGS
WORKDIR /build/cli
RUN cargo deb --no-build --deb-version $REPO_VERSION --manifest-path ./Cargo.toml

# Log the commit hash
COPY .git/ /tmp/.git/
WORKDIR /tmp
RUN git rev-parse HEAD > /commit-hash

# -------------=== gridd docker build ===-------------
FROM ubuntu:focal

ENV DEBIAN_FRONTEND=noninteractive

ARG CARGO_ARGS
RUN echo "CARGO_ARGS = '$CARGO_ARGS'" > CARGO_ARGS

COPY --from=gridd-builder /build/target/debian/grid-cli_*.deb /tmp
COPY --from=gridd-builder /build/target/debian/grid-daemon_*.deb /tmp
COPY --from=gridd-builder /commit-hash /commit-hash

# hadolint ignore=DL3015
RUN apt-get update \
 && apt-get install -y -q \
    ca-certificates \
    curl \
    libsqlite3-dev \
    man \
    postgresql-client \
 && mandb \
 && dpkg --unpack /tmp/grid*.deb \
 && apt-get -f -y install \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN echo ". /usr/share/bash-completion/bash_completion" >> ~/.bashrc

CMD ["gridd"]
