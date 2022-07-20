# Copyright 2022 Cargill Incorporated
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

FROM alpine:3

RUN apk add --no-cache curl

WORKDIR /tmp

RUN curl -O -L https://github.com/crate-ci/typos/releases/download/v1.10.2/typos-v1.10.2-x86_64-unknown-linux-musl.tar.gz \
 && tar xzvf typos-v1.10.2-x86_64-unknown-linux-musl.tar.gz

ENV PATH=$PATH:/tmp

WORKDIR /project
