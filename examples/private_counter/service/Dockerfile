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
 libssl-dev \
 libzmq3-dev \
 pkg-config \
 unzip

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
COPY protos /build/protos
COPY libsplinter /build/libsplinter

# Create empty cargo project
WORKDIR /build/examples/private_counter
RUN USER=root cargo new --bin service --name private-counter

# Copy over Cargo.toml file
COPY examples/private_counter/service/Cargo.toml /build/examples/private_counter/service/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/examples/private_counter/service
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
RUN rm src/*
RUN rm target/release/private-counter* target/release/deps/private_counter*

# Copy over source files
COPY examples/private_counter/protos /build/examples/private_counter/protos
COPY examples/private_counter/service/build.rs /build/examples/private_counter/service/build.rs
COPY examples/private_counter/service/src /build/examples/private_counter/service/src
COPY libsplinter/src /build/libsplinter/src

# Build the project
RUN cargo build --release

# Create the standalone image
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y \
 libssl1.1 \
 libzmq5 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=BUILDER /build/examples/private_counter/service/target/release/private-counter /usr/bin/private-counter

EXPOSE 8000
CMD ["private-counter"]
