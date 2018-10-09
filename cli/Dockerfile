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

FROM ubuntu:xenial

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

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

ENV PATH=$PATH:/root/.cargo/bin

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
    && unzip -o protoc-3.5.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.5.1-linux-x86_64.zip
RUN apt-get update && apt-get install -y protobuf-compiler

# Pull in deps
COPY protos/ /project/protos/
COPY messaging/ /project/messaging/
COPY client/ /project/client/

# Build with dummy source in order to cache dependencies in Docker image
WORKDIR /project
RUN USER=root cargo new --bin cli --name slinter-cli

WORKDIR /project/cli
COPY cli/Cargo.toml cli/Cargo.lock* ./
RUN cargo build

# Build full component
COPY cli/ ./
RUN cargo build

ENV PATH=$PATH:/project/cli/target/debug/


CMD ["splinter-cli"]
