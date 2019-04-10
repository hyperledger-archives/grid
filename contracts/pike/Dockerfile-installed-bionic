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

# docker build -f contracts/sawtooth-pike/tp/Dockerfile-installed-bionic -t sawtooth-pike-tp .

# -------------=== pike tp build ===-------------

FROM ubuntu:bionic as pike-tp-builder

ENV VERSION=AUTO_STRICT

RUN apt-get update \
 && apt-get install -y \
 curl \
 gcc \
 git \
 libssl-dev \
 libzmq3-dev \
 pkg-config \
 python3 \
 unzip

# For Building Protobufs
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
 && curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

ENV PATH=$PATH:/protoc3/bin
RUN /root/.cargo/bin/cargo install cargo-deb

COPY . /project

WORKDIR /project/contracts/sawtooth-pike/tp

RUN sed -i -e s/version.*$/version\ =\ \"$(../../../bin/get_version)\"/ Cargo.toml
RUN /root/.cargo/bin/cargo deb

# -------------=== pike tp docker build ===-------------

FROM ubuntu:bionic

COPY --from=pike-tp-builder /project/contracts/sawtooth-pike/tp/target/debian/pike*.deb /tmp

RUN apt-get update \
 && dpkg -i /tmp/pike*.deb || true \
 && apt-get -f -y install

ENTRYPOINT ["tail", "-f", "/dev/null"]
