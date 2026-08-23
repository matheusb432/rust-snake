set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Run the game
[group('build')]
run:
    RUSTFLAGS="-Awarnings" cargo run
