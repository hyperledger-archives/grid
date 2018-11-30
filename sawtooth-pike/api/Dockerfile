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

FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y -q \
 curl \
 gcc \
 libpq-dev \
 libssl-dev \
 libzmq3-dev \
 pkg-config \
 unzip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# For Building Protobufs
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
&& /root/.cargo/bin/rustup default nightly \
 && curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

WORKDIR /project/contracts/sawtooth-pike/api

ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin:/project/contracts/sawtooth-pike/api/target/debug/
