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

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
    && unzip -o protoc-3.5.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.5.1-linux-x86_64.zip

ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin

COPY ./sdk /sdk

RUN USER=root cargo new --bin contracts/schema
WORKDIR /contracts/schema

# Build TP with dummy source in order to cache dependencies in Docker image.
COPY ./contracts/schema/Cargo.toml ./Cargo.toml
RUN cargo build --release

COPY contracts/schema/Cargo.toml contracts/schema/Cargo.lock* ./
RUN cargo build

RUN rm src/*.rs
COPY ./contracts/schema/src ./src

RUN rm ./target/release/grid-schema-tp* ./target/release/deps/grid_schema*
RUN cargo build --release

# Create the stand-alone stage
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y libssl1.1 libzmq5 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=GRID-SCHEMA-BUILDER /contracts/schema/target/release/grid-schema-tp /

CMD ["/grid-schema-tp"]
