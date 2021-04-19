# Copyright 2018-2021 Cargill Incorporated
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

crates := '\
    sdk \
    daemon \
    griddle \
    cli \
    contracts/pike \
    contracts/location \
    contracts/product \
    contracts/schema \
    contracts/track_and_trace \
    '

features := '\
    --features=experimental \
    --features=stable \
    --features=default \
    --no-default-features \
    '

build:
    #!/usr/bin/env sh
    set -e
    for feature in $(echo {{features}})
    do
        for crate in $(echo {{crates}})
        do
            cmd="cargo build --tests --manifest-path=$crate/Cargo.toml $feature"
            echo "\033[1m$cmd\033[0m"
            $cmd
        done
    done
    echo "\n\033[92mBuild Success\033[0m\n"

build-experimental:
    #!/usr/bin/env sh
    set -e
    for crate in $(echo {{crates}})
    do
        cmd="cargo build --tests --manifest-path=$crate/Cargo.toml --features=experimental"
        echo "\033[1m$cmd\033[0m"
        $cmd
    done
    echo "\n\033[92mBuild Success\033[0m\n"

ci:
    just ci-lint-ui
    just ci-test-ui
    just ci-lint-grid
    just ci-unit-test-grid
    just ci-build-gridd-experimental

ci-build-gridd-experimental:
    #!/usr/bin/env sh
    set -e
    ISOLATION_ID=$ISOLATION_ID"experimental" \
    CARGO_ARGS=" --features experimental" \
    docker-compose -f docker-compose.yaml build gridd

ci-build-ui-test-deps:
    #!/usr/bin/env sh
    set -e
    docker build ui/grid-ui -f ui/grid-ui/docker/test/Dockerfile -t grid-ui:$ISOLATION_ID
    docker build . -f ui/saplings/product/test/Dockerfile -t product-sapling:$ISOLATION_ID

ci-lint-grid:
    #!/usr/bin/env sh
    set -e
    docker build . -f docker/lint -t lint-grid:$ISOLATION_ID
    docker run --rm -v $(pwd):/project/grid lint-grid:$ISOLATION_ID

ci-lint-ui: ci-build-ui-test-deps
    #!/usr/bin/env sh
    set -e
    docker run --rm --env CI=true grid-ui:$ISOLATION_ID yarn lint
    docker run --rm --env CI=true product-sapling:$ISOLATION_ID yarn test

ci-unit-test-grid:
    #!/usr/bin/env sh
    set -e
    REPO_VERSION=$(./bin/get_version) docker-compose -f docker-compose.yaml build --force-rm
    docker-compose -f docker/compose/grid_tests.yaml build --force-rm
    docker-compose -f docker/compose/grid_tests.yaml up --abort-on-container-exit --exit-code-from grid_tests

ci-test-ui: ci-build-ui-test-deps
    #!/usr/bin/env sh
    set -e
    docker run --rm --env CI=true grid-ui:$ISOLATION_ID yarn test
    docker run --rm --env CI=true product-sapling:$ISOLATION_ID yarn test

clean:
    cargo clean

integration-test:
    #!/usr/bin/env sh
    set -e
    ./bin/run_integration_tests

lint:
    #!/usr/bin/env sh
    set -e
    echo "\033[1mcargo fmt -- --check\033[0m"
    cargo fmt -- --check
    for feature in $(echo {{features}})
    do
        for crate in $(echo {{crates}})
        do
            cmd="cargo clippy --manifest-path=$crate/Cargo.toml $feature -- -D warnings"
            echo "\033[1m$cmd\033[0m"
            $cmd
        done
    done
    echo "\n\033[92mLint Success\033[0m\n"

lint-experimental:
    #!/usr/bin/env sh
    set -e
    echo "\033[1mcargo fmt -- --check\033[0m"
    cargo fmt -- --check
    for crate in $(echo {{crates}})
    do
        cmd="cargo clippy --manifest-path=$crate/Cargo.toml --features=experimental -- -D warnings"
        echo "\033[1m$cmd\033[0m"
        $cmd
    done
    echo "\n\033[92mLint Success\033[0m\n"

test: build
    #!/usr/bin/env sh
    set -e
    for feature in $(echo {{features}})
    do
        for crate in $(echo {{crates}})
        do
            cmd="cargo test --manifest-path=$crate/Cargo.toml $feature"
            echo "\033[1m$cmd\033[0m"
            $cmd
        done
    done
    echo "\n\033[92mTest Success\033[0m\n"

test-experimental: build-experimental
    #!/usr/bin/env sh
    set -e
    for crate in $(echo {{crates}})
    do
        cmd="cargo test --manifest-path=$crate/Cargo.toml --features=experimental"
        echo "\033[1m$cmd\033[0m"
        $cmd
    done
    echo "\n\033[92mTest Success\033[0m\n"

test-experimental-hack: build-experimental
    #!/usr/bin/env sh
    set -e
    for crate in $(echo {{crates}})
    do
        cmd="cargo test --manifest-path=$crate/Cargo.toml --features=experimental -- --test-threads=1"
        echo "\033[1m$cmd\033[0m"
        $cmd
    done
    echo "\n\033[92mTest Success\033[0m\n"
