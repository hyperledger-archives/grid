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

FROM ubuntu:jammy

ENV DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Install base dependencies
RUN apt-get update \
 && apt-get install -y -q --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    g++ \
    gcc \
    git \
    libpq-dev \
    libsasl2-dev \
    libsqlite3-dev \
    libssl-dev \
    libxml2-dev \
    libzmq3-dev \
    openssl \
    pandoc \
    pkg-config \
    unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

ENV PATH=$PATH:/root/.cargo/bin

# Install Rust
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y \
 && rustup update \
 && rustup target add wasm32-unknown-unknown \
# Install cargo deb
 && cargo install cargo-deb \
# Install protoc
 && TARGET_ARCH=$(dpkg --print-architecture) \
 && if [[ $TARGET_ARCH == "arm64" ]]; then \
      PROTOC_ARCH="aarch_64"; \
    elif [[ $TARGET_ARCH == "amd64" ]]; then \
      PROTOC_ARCH="x86_64"; \
    fi \
 && curl -OLsS https://github.com/google/protobuf/releases/download/v3.20.0/protoc-3.20.0-linux-$PROTOC_ARCH.zip \
      && unzip -o protoc-3.20.0-linux-$PROTOC_ARCH.zip -d /usr/local \
      && rm protoc-3.20.0-linux-$PROTOC_ARCH.zip

# Install just
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin

# Create empty cargo projects for top-level projects
WORKDIR /build
RUN USER=root cargo new --bin cli \
 && USER=root cargo new --bin daemon \
 && USER=root cargo new --bin griddle \
 && USER=root cargo new --lib sdk \
# Create empty Cargo projects for contracts
 && USER=root cargo new --bin contracts/location \
 && USER=root cargo new --bin contracts/pike \
 && USER=root cargo new --bin contracts/product \
 && USER=root cargo new --bin contracts/purchase_order \
 && USER=root cargo new --bin contracts/schema \
 && USER=root cargo new --bin contracts/track_and_trace

# Copy over Cargo.toml files
COPY Cargo.toml /build/Cargo.toml
COPY cli/Cargo.toml /build/cli/Cargo.toml
COPY daemon/Cargo.toml /build/daemon/Cargo.toml
COPY griddle/Cargo.toml /build/griddle/Cargo.toml
COPY sdk/Cargo.toml /build/sdk/Cargo.toml

COPY contracts/location/Cargo.toml /build/contracts/location/Cargo.toml
COPY contracts/pike/Cargo.toml /build/contracts/pike/Cargo.toml
COPY contracts/product/Cargo.toml /build/contracts/product/Cargo.toml
COPY contracts/purchase_order/Cargo.toml /build/contracts/purchase_order/Cargo.toml
COPY contracts/schema/Cargo.toml /build/contracts/schema/Cargo.toml
COPY contracts/track_and_trace/Cargo.toml /build/contracts/track_and_trace/Cargo.toml

# Do release builds for each Cargo.toml
# Workaround for https://github.com/koalaman/shellcheck/issues/1894
#hadolint ignore=SC2016
RUN find ./*/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=experimental' sh {} \
 && find ./*/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=stable' sh {} \
 && find ./*/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=default' sh {} \
 && find ./*/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --no-default-features' sh {} \
# Do wasm builds for the contracts
 && find ./contracts/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=experimental' sh {} \
 && find ./contracts/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=stable' sh {} \
 && find ./contracts/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --features=default' sh {} \
 && find ./contracts/ -name 'Cargo.toml' -print0 | \
    xargs -0 -I {} sh -c 'echo Building $1; cargo build --tests --release --manifest-path $1 --no-default-features' sh {} \
# Clean up built files
 && rm -f \
    target/release/grid* \
    target/release/deps/grid* \
    target/wasm32-unknown-unknown/release/grid* \
    target/wasm32-unknown-unknown/release/deps/grid* \
# Clean up leftover files
 && find . -name 'Cargo.toml' -exec \
    sh -c 'x="$1"; rm "$x" ' sh {} \;
