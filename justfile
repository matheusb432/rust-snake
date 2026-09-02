set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Run the game
[group('build')]
up:
    RUSTFLAGS="-Awarnings" cargo run

[group('quality')]
fmt *args:
  cargo fmt

[group('quality')] 
test *args:
  cargo nextest run "$@"
