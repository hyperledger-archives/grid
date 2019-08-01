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

FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y \
    curl \
    gcc \
    g++ \
    libpq-dev \
    libssl-dev \
    libzmq3-dev \
    openssl \
    pkg-config \
    unzip \
    postgresql-client \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.7.1/protoc-3.7.1-linux-x86_64.zip \
    && unzip -o protoc-3.7.1-linux-x86_64.zip -d /usr/local \
    && rm protoc-3.7.1-linux-x86_64.zip

# Copy over libsplinter, protos and create the example folder, for the
# relative dependencies
COPY ./protos /protos
COPY ./libsplinter /libsplinter

COPY /examples/gameroom /examples/gameroom

WORKDIR /examples/gameroom/daemon
RUN cargo build

ENV PATH=$PATH:/project/gameroom/daemon/target/debug/
