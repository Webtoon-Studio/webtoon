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

Welcome to the `webtoon` library, a Rust-based SDK that allows you to interact with a Webtoon platform programmatically.

This library provides a set of utilities and methods to handle various Webtoon-specific operations such as fetching episodes,
posting comments, subscribing, liking, and managing episode metadata. Platform support varies.

Supported:
- [webtoons.com](https://www.webtoons.com/) (language support varies)
- [comic.naver.com](https://comic.naver.com/)
- More to come!

### Capabilities

- Fetch information about webtoons and their episodes and their posts.
- Subscribe/unsubscribe to webtoons(`webtoons.com` only).
- Like/unlike episodes (`webtoons.com` only).
- Post and manage comments(`webtoons.com` only).
- Retrieve detailed episode information such as views, published status, season number, etc.

### Installation

MSRV: `1.85.0`

To use this library, add `webtoon` to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
webtoon = "0.9.0"
```

## Quick-Start

The main entry point to the library is through a `Client`. Each platform has its own client that is responsible for
exposing various ways to interact with the specific platform.

### `webtoons.com`

```rust
use webtoon::platform::webtoons::{errors::Error, Client, Type};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the client
    let client = Client::new();

    // Fetch a webtoon by its `id` and its `Type`
    let webtoon = client
        .webtoon(95, Type::Original)
        .await?
        .expect("No webtoon with this id and type on webtoon.com");

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
        .expect("No webtoon with this id on comic.naver.com");

    // Print title to stdout
    println!("{}", webtoon.title());

    Ok(())
}
```

For more examples, check out the [`examples`](https://github.com/Webtoon-Studio/webtoon/tree/main/examples) folder.

## Features
In an effort to reduce compile times, dependency trees, and binary sizes, there are some pieces of functionality that are locked behind features.

- `rss`: Enables the ability to get the RSS feed data for a `webtoons.com`.
- `naver`: Enables the ability to interact with `comic.naver.com`.
- `download`: Enables the ability to download episodes, either as a single image or as multiple images.
