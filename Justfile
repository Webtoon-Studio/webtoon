test t="" $RUSTFLAGS="-Zthreads=8 -Cforce-frame-pointers" $RUST_BACKTRACE="1":
    cargo +nightly test {{t}} --all-features --no-fail-fast --profile test --all

smoke $RUSTFLAGS="-Zthreads=8 -Cforce-frame-pointers" $RUST_BACKTRACE="1":
    cargo +nightly test --all-features --profile test --test webtoons_smoke -- --include-ignored --no-capture --test-threads 1

clippy $RUSTFLAGS="-Zthreads=8":
    cargo +nightly clippy --all-features

doc $RUSTDOCFLAGS="--cfg docsrs":
    cargo +nightly doc --all-features --no-deps

cliff:
    git cliff --bump -o
