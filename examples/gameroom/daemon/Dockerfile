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

RUN apt-get update \
 && apt-get install -y \
    curl \
    gcc \
    g++ \
    libpq-dev \
    libssl-dev \
    libzmq3-dev \
    openssl \
    pkg-config \
    unzip \
    postgresql-client \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

ENV PATH=$PATH:/root/.cargo/bin

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

# Install protoc
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip \
    && unzip -o protoc-3.7.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.7.1-linux-x86_64.zip

# Copy dependencies
COPY examples/gameroom/database /build/examples/gameroom/database
COPY protos /build/protos

# Create empty cargo project for libsplinter
WORKDIR /build
RUN USER=root cargo new --lib libsplinter

# Copy over Cargo.toml and build.rs
COPY libsplinter/build.rs /build/libsplinter/build.rs
COPY libsplinter/Cargo.toml /build/libsplinter/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/libsplinter
RUN cargo build --release

# Create empty cargo project for gameroomd
WORKDIR /build/examples/gameroom
RUN USER=root cargo new --bin daemon

# Copy over gameroomd Cargo.toml file
COPY examples/gameroom/daemon/Cargo.toml /build/examples/gameroom/daemon/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/examples/gameroom/daemon
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
WORKDIR /build
RUN rm */src/*.rs
RUN rm examples/gameroom/daemon/target/release/gameroom* \
    examples/gameroom/daemon/target/release/deps/gameroom* \
    examples/gameroom/daemon/target/release/deps/*libsplinter*

# Copy over source files
COPY examples/gameroom/daemon/src /build/examples/gameroom/daemon/src
COPY libsplinter/ /build/libsplinter/
COPY protos/ /build/protos

# Build the project
WORKDIR /build/examples/gameroom/daemon
RUN cargo build --release

# create the standalone image
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y \
    libssl1.1 \
    libzmq5 \
    postgresql-client \
    curl \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=BUILDER /build/examples/gameroom/daemon/target/release/gameroomd /usr/bin/gameroomd
