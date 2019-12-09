# Copyright (c) 2019 Target Brands, Inc.
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

FROM hyperledger/grid-dev as grid-product-builder

COPY contracts/product /build/contracts/product
COPY sdk/ /build/sdk/

# Do a release build to cache dependencies
WORKDIR /build/contracts/product

# Build the contract
RUN cargo build --target wasm32-unknown-unknown --release

# -------------=== product submitter build ===-------------

FROM hyperledger/sawtooth-sabre-cli

COPY --from=grid-product-builder \
     /build/target/wasm32-unknown-unknown/release/grid-product-tp.wasm /tmp

# Copy the contract definition
COPY contracts/product/product.yaml /tmp
