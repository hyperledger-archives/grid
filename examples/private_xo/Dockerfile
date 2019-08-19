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
 && apt-get install -y \
    curl \
    gcc \
    libssl-dev \
    libzmq3-dev \
    pkg-config \
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
COPY protos /build/protos

# Create empty cargo project for libsplinter
WORKDIR /build
RUN USER=root cargo new --lib libsplinter

# Copy over libsplinter Cargo.toml and build.rs
COPY libsplinter/build.rs /build/libsplinter/build.rs
COPY libsplinter/Cargo.toml /build/libsplinter/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/libsplinter
RUN cargo build --release

# Create empty cargo for private_xo
WORKDIR /build/examples
RUN USER=root cargo new --bin private_xo

# Copy over private_xo Cargo.toml file
COPY examples/private_xo/Cargo.toml /build/examples/private_xo/Cargo.toml

# Do a release build to cache dependencies
WORKDIR /build/examples/private_xo
RUN cargo build --release

# Remove the auto-generated .rs files and the built files
RUN rm src/*.rs
RUN rm target/release/private-xo* target/release/deps/private_xo* \
   target/release/deps/*libsplinter*

# Copy over source files
COPY ./examples/private_xo/protos /build/examples/private_xo/protos
COPY ./examples/private_xo/build.rs /build/examples/private_xo/build.rs
COPY ./examples/private_xo/src /build/examples/private_xo/src
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

COPY --from=BUILDER /build/examples/private_xo/target/release/private-xo /usr/bin/private-xo

EXPOSE 8000

CMD ["private-xo"]
