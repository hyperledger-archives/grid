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
    contracts/purchase_order \
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
    just ci-lint
    just ci-test
    just ci-test-integration

ci-build-ui-test-deps:
    #!/usr/bin/env sh
    set -e
    docker build . -f ui/grid-ui/docker/test/Dockerfile -t grid-ui:$ISOLATION_ID
    docker build . -f ui/saplings/product/test/Dockerfile -t product-sapling:$ISOLATION_ID

ci-lint:
    #!/usr/bin/env sh
    set -e
    docker-compose -f docker/compose/run-lint.yaml build lint-grid
    docker-compose -f docker/compose/run-lint.yaml up \
      --abort-on-container-exit lint-grid

ci-lint-ui: ci-build-ui-test-deps
    #!/usr/bin/env sh
    set -e
    docker run --rm --env CI=true grid-ui:$ISOLATION_ID just lint-grid-ui
    docker run --rm --env CI=true product-sapling:$ISOLATION_ID just lint-product-sapling

ci-test:
    #!/usr/bin/env sh
    set -e
    docker-compose -f docker/compose/grid-tests.yaml build --force-rm
    docker-compose -f docker/compose/grid-tests.yaml up --abort-on-container-exit --exit-code-from grid_tests

ci-test-integration:
    #!/usr/bin/env sh
    set -e
    echo "\033[1mRunning daemon integration test\033[0m"
    cd daemon/test && \
      CARGO_ARGS=" --features experimental" \
      docker-compose up \
        --abort-on-container-exit \
        --exit-code-from daemon --build
    echo "\033[1mRunning integration test\033[0m"
    cd ../../integration && \
      CARGO_ARGS=" --features experimental" \
      docker-compose up \
        --abort-on-container-exit \
        --exit-code-from gridd --build

ci-test-ui: ci-build-ui-test-deps
    #!/usr/bin/env sh
    set -e
    docker run --rm --env CI=true grid-ui:$ISOLATION_ID just test-grid-ui
    docker run --rm --env CI=true product-sapling:$ISOLATION_ID just test-product-sapling

clean:
    cargo clean

copy-env:
    #!/usr/bin/env sh
    set -e
    find . -name .env | xargs -I '{}' sh -c "echo 'Copying to {}'; rsync .env {}"

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

lint-grid-ui:
    #!/usr/bin/env sh
    set -e
    cd ui/grid-ui
    yarn lint
    echo "\n\033[92mLint Grid UI Success\033[0m\n"

lint-product-sapling:
    #!/usr/bin/env sh
    set -e
    cd ui/saplings/product
    yarn lint
    echo "\n\033[92mLint Product Sapling Success\033[0m\n"

lint-ui:
    just lint-grid-ui
    just lint-product-sapling

test: build
    #!/usr/bin/env sh
    set -e
    for feature in $(echo {{features}})
    do
        for crate in $(echo {{crates}})
        do
            cmd="cargo test --manifest-path=$crate/Cargo.toml $feature $TEST_ARGS"
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

test-grid-ui:
    #!/usr/bin/env sh
    set -e
    cd ui/grid-ui
    yarn test
    echo "\n\033[92mTest Grid UI Success\033[0m\n"

test-product-sapling:
    #!/usr/bin/env sh
    set -e
    cd ui/saplings/product
    yarn test
    echo "\n\033[92mTest Product Sapling Success\033[0m\n"

test-ui:
    just test-grid-ui
    just test-product-sapling
