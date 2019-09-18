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
    python \
    unzip \
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
COPY protos/ /build/protos/

# Create empty cargo project for libsplinter
WORKDIR /build
RUN USER=root cargo new --lib libsplinter

# Copy over Cargo.toml and build.rs
COPY libsplinter/build.rs /build/libsplinter/build.rs
COPY libsplinter/Cargo.toml /build/libsplinter/Cargo.toml

# Do a release build to cache libsplinter dependencies
WORKDIR /build/libsplinter
RUN cargo build --release

# Create empty cargo project for cli
WORKDIR /build
RUN USER=root cargo new --bin cli

# Copy over Cargo.toml file
COPY cli/Cargo.toml /build/cli/Cargo.toml

# Do a release build to cache cli dependencies
WORKDIR /build/cli
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
WORKDIR /build
RUN rm cli/target/release/deps/*splinter*
RUN rm libsplinter/target/release/*splinter* \
    libsplinter/target/release/deps/*splinter*

# Copy over source files
COPY cli/src /build/cli/src
COPY libsplinter/src /build/libsplinter/src
COPY protos/ /build/protos/

# Build the project
WORKDIR /build/cli
RUN cargo build --release

# create the standalone image
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y \
    libssl1.1 \
    libzmq5 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=BUILDER /build/cli/target/release/splinter-cli /usr/bin/splinter-cli

CMD ["splinter-cli"]
