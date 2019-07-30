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

FROM ubuntu:bionic as BUILDER

# Install base dependencies
RUN apt-get update \
 && apt-get install -y -q \
    build-essential \
    curl \
    gcc \
    g++ \
    libpq-dev \
    libssl-dev \
    libsasl2-dev \
    libzmq3-dev \
    openssl \
    pkg-config \
    unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

# Install protoc
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip \
 && unzip -o protoc-3.7.1-linux-x86_64.zip -d /usr/local \
 && rm protoc-3.7.1-linux-x86_64.zip

ENV PATH=$PATH:/root/.cargo/bin

RUN mkdir /build

# Copy grid sdk dependency
COPY sdk/ /build/sdk/

# Create empty cargo projects
WORKDIR /build
RUN USER=root cargo new --bin daemon
RUN USER=root cargo new --bin cli

# Copy over Cargo.toml files
COPY daemon/Cargo.toml /build/daemon/Cargo.toml
COPY cli/Cargo.toml /build/cli/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/daemon
RUN cargo build --release
WORKDIR /build/cli
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
WORKDIR /build
RUN rm */src/*.rs
RUN rm daemon/target/release/gridd* daemon/target/release/deps/gridd*
RUN rm cli/target/release/grid* cli/target/release/deps/grid*

# Copy over source files
COPY sdk/src /build/sdk/src
COPY daemon/src /build/daemon/src
COPY daemon/migrations /build/daemon/migrations
COPY cli/src /build/cli/src

# Build the projects
WORKDIR /build/daemon
RUN cargo build --release
WORKDIR /build/cli
RUN cargo build --release

# create the standalone image
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y libssl1.1 libzmq5 libpq5 postgresql-client \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=BUILDER /build/daemon/target/release/gridd /usr/sbin/gridd
COPY --from=BUILDER /build/cli/target/release/grid /usr/bin/grid

CMD ["gridd"]
