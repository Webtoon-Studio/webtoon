set windows-shell := ["pwsh", "-c"]

test:
    cargo test --all-features --no-fail-fast --release --all

cliff:
    git cliff --bump -o
