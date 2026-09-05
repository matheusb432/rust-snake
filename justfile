set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Run the game
[group('build')]
up:
    RUSTFLAGS="-Awarnings" cargo run

# TODO: add for release build
# [group('release')]
# play:
#   ..

[group('quality')]
fmt *args:
  cargo fmt

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
    mise bootstrap --yes {{ args }}

# Fetch the Rust dependencies for this checkout.
[group('setup')]
setup *args:
    cargo fetch {{ args }}
