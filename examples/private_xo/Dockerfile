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

RUN apt-get update \
 && apt-get install -y \
 curl \
 gcc \
 libssl-dev \
 libzmq3-dev \
 pkg-config \
 unzip

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=$PATH:/protoc3/bin:/root/.cargo/bin

RUN rustup install nightly && \
    rustup default nightly

# Copy over libsplinter, protos and create the example folder, for the
# relative dependencies
COPY ./protos /protos
COPY ./libsplinter /libsplinter
RUN mkdir -p /examples

# Create an empty project, copy over the Cargo toml, and buld the deps first
WORKDIR /examples
RUN USER=root cargo new --bin private-xo
WORKDIR /examples/private-xo

COPY ./examples/private_xo/Cargo.toml ./Cargo.toml
RUN cargo build --release

# Copy over the source and build the app
RUN rm src/*.rs
COPY ./examples/private_xo/src ./src

RUN rm ./target/release/private-xo* ./target/release/deps/private_xo*
RUN cargo build --release

# Create the standalone image
FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install -y libssl1.1 libzmq5 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=BUILDER /examples/private-xo/target/release/private-xo /

EXPOSE 8000
CMD ["/private-xo"]
