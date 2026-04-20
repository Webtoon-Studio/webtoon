<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/webtoon">
    <img src="https://img.shields.io/crates/v/webtoon.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Docs -->
  <a href="https://docs.rs/webtoon">
  <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/webtoon">
    <img src="https://img.shields.io/crates/d/webtoon.svg?style=flat-square" alt="Download" />
  </a>
</div>

# Webtoon

`webtoon` is an asynchronous Rust library for programmatically interacting with
supported Webtoon platforms. It provides a strongly typed, idiomatic API for
retrieving metadata, episodes, and discussion data, while abstracting away
platform-specific quirks.

Supported:

- [webtoons.com](https://www.webtoons.com/)
- [comic.naver.com](https://comic.naver.com/)

## Design Philosophy: Read-Only

This crate is built as a data access layer. It is intentionally read-only to
ensure reliability and avoid the risks associated with automated interaction
(likes, votes, or posting).

This makes it ideal for:

- Community analysis and metrics tracking.
- Research and data archival.
- Creator tools (e.g., [`scrapetoon`](https://github.com/RoloEdits/scrapetoon))

### Installation

MSRV: `1.88.0`

To use this library, add `webtoon` to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
webtoon = "0.10.0"
```

## Quick-Start

The primary entry point is a platform-specific Client. Each client encapsulates
authentication, request handling, and platform behavior.

### `webtoons.com`

```rust
use webtoon::platform::webtoons::{error::Error, Client, Type};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the client
    let client = Client::new();

    // Fetch a webtoon by its `id` and its `Type`
    let webtoon = client
        .webtoon(95, Type::Original)
        .await?
        .expect("known webtoon with this id and type on `webtoons.com`");

    // Fetch title and print to stdout
    println!("{}", webtoon.title().await?);

    Ok(())
}
```

### `comic.naver.com`

```rust
use webtoon::platform::naver::{errors::Error, Client};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the client
    let client = Client::new();

    // Fetch a webtoon by `id`
    let webtoon = client
        .webtoon(838432)
        .await?
        .expect("known webtoon with this id on `comic.naver.com`");

    // Print title to stdout
    println!("{}", webtoon.title());

    Ok(())
}
```

## Features

Some functionality is gated behind Cargo features to reduce compile times and
dependency overhead:

- `rss`: Enables the ability to get the RSS feed data for a `webtoons.com`.
- `naver`: Enables the ability to interact with `comic.naver.com`.
- `download`: Enables the ability to download episodes, either as a single image or as multiple images.

```toml
webtoon = { version = "0.10.0", features = ["naver", "download"] }
```

## Examples

For more examples, check out the [`examples`](https://github.com/Webtoon-Studio/webtoon/tree/main/examples) folder.
