set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set positional-arguments

_default:
    @just --list --unsorted

# Run the game
[group('build')]
up:
    cargo run

[group('release')]
play:
    cargo run --release

[group('quality')]
fmt *args:
    cargo fmt --all "$@"

[group('quality')]
fmt-check:
    cargo fmt --all --check

[group('quality')]
lint:
    cargo clippy --workspace --all-targets -- -D warnings

[group('quality')]
check: fmt-check lint

[group('quality')]
fix *args:
    cargo clippy --fix --workspace --all-targets --allow-dirty "$@"
    just fmt

[group('quality')]
test *args:
    cargo nextest run "$@"

# Report missing tools and native packages without installing them.
[group('setup')]
doctor:
    @mise ls --local --missing --locked --no-header
    @mise bootstrap packages status --missing

# Install the declared native packages and pinned tools, then prepare the checkout.
[group('setup')]
bootstrap *args:
    mise bootstrap --yes "$@"

# Fetch the Rust dependencies for this checkout.
[group('setup')]
setup *args:
    cargo fetch "$@"
