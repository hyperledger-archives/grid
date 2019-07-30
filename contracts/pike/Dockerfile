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

FROM ubuntu:bionic as grid-pike-builder

RUN apt-get update \
&& apt-get install gnupg -y

# Install base dependencies
RUN echo "deb http://repo.sawtooth.me/ubuntu/nightly bionic universe" >> /etc/apt/sources.list \
 && (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 44FC67F19B2466EA \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 44FC67F19B2466EA) \
 && apt-get update \
 && apt-get install -y -q \
    curl \
    gcc \
    libssl-dev \
    libzmq3-dev \
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
 && rustup target add wasm32-unknown-unknown

# Install protoc
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip \
 && unzip -o protoc-3.7.1-linux-x86_64.zip -d /usr/local \
 && rm protoc-3.7.1-linux-x86_64.zip

RUN mkdir /build

# Copy grid sdk dependency
COPY sdk/ /build/sdk/

# Create empty cargo project
WORKDIR /build
RUN mkdir contracts \
 && USER=root cargo new --bin contracts/pike

# Copy over Cargo.toml file
COPY contracts/pike/Cargo.toml /build/contracts/pike/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/contracts/pike
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
RUN rm src/*.rs
RUN rm target/release/grid-pike-tp* target/release/deps/grid_pike*

# Copy over source files
COPY contracts/pike/src /build/contracts/pike/src

# Build the contract
RUN cargo build --target wasm32-unknown-unknown --release

# Copy the contract definition
COPY contracts/pike/pike.yaml /build/contracts/pike/pike.yaml

ENTRYPOINT []
