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
posting comments, subscribing, liking, and managing episode metadata.

- Currently only [webtoons.com](https://www.webtoons.com/) is supported.
- More is planned!

### Features

- Fetch information about webtoons and their episodes.
- Subscribe/unsubscribe to webtoons.
- Like/unlike episodes.
- Post and manage comments.
- Retrieve detailed episode information such as views, published status, season number, etc.

### Installation

To use this library, add `webtoon` to your `Cargo.toml`:

```toml
[dependencies]
webtoon = "0.2.3"
```

## Example Usage

### `webtoons.com`

```rust
use webtoon::platform::webtoons::{errors::Error, Client, Type};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the client
    let client = Client::new();
    
    // Fetch a webtoon by ID and Type
    let webtoon = client
        .webtoon(95, Type::Original)
        .await?
        .expect("No webtoon with this id and type on webtoon.com");
    
    // Fetch title and print to stdout
    println!("{}", webtoon.title().await?);

    Ok(())
}
```

For more examples, check out the [`examples`](https://github.com/Webtoon-Studio/webtoon/tree/main/examples) folder.

## Features

- `rss`: Enables the ability to get the RSS feed data for a webtoon.
- `download`: Enables the ability to download an episodes panels.
