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
 && apt-get install -y -q \
 curl \
 gcc \
 libssl-dev \
 libzmq3-dev \
 pkg-config \
 unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

# For Building Protobufs
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
 && curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin

COPY ./sdk /sdk

RUN USER=root cargo new --bin contracts/pike
WORKDIR /contracts/pike

# Build TP with dummy source in order to cache dependencies in Docker image.
COPY ./contracts/pike/Cargo.toml ./Cargo.toml
RUN cargo build --release

COPY contracts/pike/Cargo.toml contracts/pike/Cargo.lock* ./
RUN cargo build

RUN rm src/*.rs
COPY ./contracts/pike/src ./src

RUN rm ./target/release/grid-pike-tp* ./target/release/deps/grid_pike*
RUN cargo build --release

# Create the stand-alone stage
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y libssl1.1 libzmq5 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=grid-pike-builder /contracts/pike/target/release/grid-pike-tp /

CMD ["/grid-pike-tp"]
