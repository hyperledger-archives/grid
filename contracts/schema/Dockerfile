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

FROM ubuntu:bionic as GRID-SCHEMA-BUILDER

RUN apt-get update \
&& apt-get install gnupg -y

# Install base dependencies
RUN echo "deb http://repo.sawtooth.me/ubuntu/nightly bionic universe" >> /etc/apt/sources.list \
 && (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 44FC67F19B2466EA \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 44FC67F19B2466EA) \
 && apt-get update \
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
        sabre-cli \
        unzip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

RUN rustup update \
 && rustup default nightly \
 && rustup target add wasm32-unknown-unknown --toolchain nightly

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
    && unzip -o protoc-3.5.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.5.1-linux-x86_64.zip

COPY ./sdk /sdk

RUN mkdir contracts \
 && USER=root cargo new --bin contracts/schema
WORKDIR /contracts/schema

# Build TP with dummy source in order to cache dependencies in Docker image.
COPY ./contracts/schema/Cargo.toml ./Cargo.toml
RUN cargo build --release

COPY contracts/schema/Cargo.toml contracts/schema/Cargo.lock* ./
RUN cargo build

RUN rm src/*.rs
COPY ./contracts/schema/src ./src

RUN rm ./target/release/grid-schema-tp* ./target/release/deps/grid_schema*
RUN cargo build --target wasm32-unknown-unknown --release

COPY ./contracts/schema/schema.yaml ./schema.yaml

ENTRYPOINT []
