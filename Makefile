# TODO: replace with root Cargo.toml?

# Quick way to build all sub-projects (expects GNU find)
.PHONY: build_all
build_all:
	find . -name "Cargo.toml" -execdir cargo build ";"

# Quick way to test all sub-projects (expects GNU find)
.PHONY: test_all
test_all:
	find . -name "Cargo.toml" -execdir cargo test ";"

# Quick way to lint all sub-projects (expects GNU find and clippy installed)
.PHONY: clip_all
clip_all:
	find . -name "Cargo.toml" -execdir cargo clippy ";"

# Quick way to lint all sub-projects (expects GNU find)
.PHONY: fmt_all
fmt_all:
	find . -name "Cargo.toml" -execdir cargo fmt ";"

