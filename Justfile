set windows-shell := ["pwsh", "-c"]

test $RUSTFLAGS="-Zthreads=8":
    cargo +nightly test --all-features --no-fail-fast --release --all

clippy $RUSTFLAGS="-Zthreads=8":
    cargo +nightly clippy --all-features

doc $RUSTDOCFLAGS="--cfg docsrs":
    cargo +nightly doc --all-features --no-deps

cliff:
    git cliff --bump -o
