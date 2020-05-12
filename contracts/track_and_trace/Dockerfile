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

FROM hyperledger/grid-dev:v2 as grid-track_and_trace-builder

# Copy over Cargo.toml files
COPY Cargo.toml /build/Cargo.toml
COPY cli/Cargo.toml /build/cli/Cargo.toml
COPY daemon/Cargo.toml /build/daemon/Cargo.toml
COPY sdk/Cargo.toml /build/sdk/Cargo.toml

COPY contracts/pike/Cargo.toml /build/contracts/pike/Cargo.toml
COPY contracts/product/Cargo.toml /build/contracts/product/Cargo.toml
COPY contracts/schema/Cargo.toml /build/contracts/schema/Cargo.toml
COPY contracts/track_and_trace/Cargo.toml /build/contracts/track_and_trace/Cargo.toml

# Copy over build files
COPY contracts/track_and_trace /build/contracts/track_and_trace
COPY sdk/ /build/sdk/

# Do a release build to cache dependencies
WORKDIR /build/contracts/track_and_trace

# Build the contract
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" Cargo.toml
RUN cargo build --target wasm32-unknown-unknown --release

# Build a scar file
# Copy the packaging directory
COPY contracts/track_and_trace/packaging/scar/* \
     /build/contracts/track_and_trace/packaging/scar/

# Copy the contract to the packaging directory
RUN cp /build/target/wasm32-unknown-unknown/release/grid-track-and-trace-tp.wasm \
    packaging/scar

WORKDIR /build/contracts/track_and_trace/packaging/scar

# Create .scar file
RUN tar -jcvf /tmp/grid-track-and-trace_${REPO_VERSION}.scar .

# -------------=== track_and_trace submitter build ===-------------

FROM hyperledger/sawtooth-sabre-cli

COPY --from=grid-track_and_trace-builder \
     /build/target/wasm32-unknown-unknown/release/grid-track-and-trace-tp.wasm /tmp

# Copy the contract definition
COPY contracts/track_and_trace/track_and_trace.yaml /tmp

COPY --from=grid-track_and_trace-builder \
    /tmp/grid-track-and-trace*.scar /tmp
