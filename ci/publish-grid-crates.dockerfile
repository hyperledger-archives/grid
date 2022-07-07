# Copyright 2021 Cargill Incorporated
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

# Description:
#   Builds an environment to publish grid-sdk to crates.io.
#   Your crates api token must be passed in as CARGO_CRED at runtime
#   using Docker's -e option.

# docker build -f ci/publish-grid-crates.dockerfile -t publish-grid-crates ci/
# docker run --rm -v $(pwd):/project/grid -e CARGO_CRED=%CREDVALUE% publish-grid-crates

FROM ubuntu:jammy

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    curl \
    gcc \
    libssl-dev \
    libzmq3-dev \
    pkg-config \
    unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

# Install protoc
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip \
    && unzip -o protoc-3.7.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.7.1-linux-x86_64.zip

ENV PATH=$PATH:/root/.cargo/bin

WORKDIR /project/grid/sdk

ARG REPO_VERSION

# hadolint ignore=DL3025
CMD cargo login $CARGO_CRED \
 && echo "Publshing version $REPO_VERSION" \
 && sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" Cargo.toml \
 && rm -f ../Cargo.lock ./Cargo.lock \
 && cargo clean \
 && cargo test \
 && cargo publish
