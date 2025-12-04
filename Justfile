test $RUSTFLAGS="-Zthreads=8":
    cargo +nightly test --all-features --no-fail-fast --profile test --all

clippy $RUSTFLAGS="-Zthreads=8":
    cargo +nightly clippy --all-features

doc $RUSTDOCFLAGS="--cfg docsrs":
    cargo +nightly doc --all-features --no-deps

cliff:
    git cliff --bump -o
