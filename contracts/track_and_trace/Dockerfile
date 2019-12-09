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

FROM hyperledger/grid-dev as grid-track_and_trace-builder

COPY contracts/track_and_trace /build/contracts/track_and_trace
COPY sdk/ /build/sdk/

# Do a release build to cache dependencies
WORKDIR /build/contracts/track_and_trace

# Build the contract
RUN cargo build --target wasm32-unknown-unknown --release

# -------------=== track_and_trace submitter build ===-------------

FROM hyperledger/sawtooth-sabre-cli

COPY --from=grid-track_and_trace-builder \
     /build/target/wasm32-unknown-unknown/release/grid-track-and-trace-tp.wasm /tmp

# Copy the contract definition
COPY contracts/track_and_trace/track_and_trace.yaml /tmp
