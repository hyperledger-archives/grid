# Copyright 2018-2022 Cargill Incorporated
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

group "default" {
    targets = [
    "gridd",
    "griddle",
    "grid-cli",
    "grid-ui",
    ]
}

# --== variables ==--

variable "CARGO_ARGS" {
    default = ""
}

variable "DISTRO" {
    default = "jammy"
}

variable "ISOLATION_ID" {
    default = "latest"
}

variable "NAMESPACE" {
    default = ""
}

variable "REGISTRY" {
    default = ""
}

variable "REPO_VERSION" {
    default = "0.4.1-dev"
}

target "all" {
    args = {
        CARGO_ARGS = "${CARGO_ARGS}"
        REPO_VERSION = "${REPO_VERSION}"
    }
    platforms = ["linux/amd64", "linux/arm64"]
}

target "gridd" {
    inherits = ["all"]
    dockerfile = "daemon/Dockerfile"
    tags = ["${REGISTRY}${NAMESPACE}gridd:${ISOLATION_ID}"]
}

target "griddle" {
    inherits = ["all"]
    dockerfile = "griddle/Dockerfile"
    tags = ["${REGISTRY}${NAMESPACE}griddle:${ISOLATION_ID}"]
}

target "grid-cli" {
    inherits = ["all"]
    dockerfile = "cli/Dockerfile"
    tags = ["${REGISTRY}${NAMESPACE}grid-cli:${ISOLATION_ID}"]
}

target "grid-ui" {
    inherits = ["all"]
    dockerfile = "ui/Dockerfile"
    tags = ["${REGISTRY}${NAMESPACE}grid-ui:${ISOLATION_ID}"]
}
