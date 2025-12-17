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

`webtoon` is an asynchronous Rust library for programmatically interacting with supported Webtoon platforms. It provides
a strongly typed, idiomatic API for retrieving metadata, episodes, and discussion data (comments and replies), while
abstracting away platform-specific quirks.

Supported:
- [webtoons.com](https://www.webtoons.com/)
- [comic.naver.com](https://comic.naver.com/)

## Goals and Intended Use

The primary goal of this library is to provide **structured, read-only access** to Webtoon platform data for use in:

- Discussion tooling and community analysis
- Data analysis and research
- Applications that assist creators (metrics, tracking, external tooling)

This crate is intentionally designed as a **data access layer**, suitable for
building higher-level tools on top of it. A real-world example of this approach
is [`scrapetoon`](https://github.com/RoloEdits/scrapetoon), which uses `webtoon`
to collect and analyze Webtoon data.

### Capabilities

- Fetch webtoon metadata (title, authors, status, etc.).
- Enumerate and inspect episodes, including:
  - Episode number and season
  - Publish status and timestamps
  - View counts and other statistics
- Access episode discussion data:
  - Lazily iterate over top-level comments
  - Retrieve replies for a given comment
  - Inspect post metadata such as votes, poster identity, and timestamps
- Platform-specific extensions exposed behind feature flags.

## Removed Interaction APIs

Earlier versions of this library exposed limited *interaction* functionality,
such as liking episodes or upvoting comments. This support has since been
removed.

The removal was driven by several factors:

- Ambiguity around automated usage (e.g. bots or spam abuse)
- Functionality falling outside the core scope of the library
- Increased maintenance burden and difficulty testing reliably
- Higher risk of platform-side breakage or enforcement

At present, `webtoon` is intentionally **read-only**. No APIs exist for
performing user actions such as voting, liking, or posting.

This decision may be revisited in the future if a clear, responsible use case
emerges, but interaction functionality is currently considered out of scope.

### Installation

MSRV: `1.88.0`

To use this library, add `webtoon` to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
webtoon = "0.9.0"
```

## Quick-Start

The primary entry point is a platform-specific Client. Each client encapsulates authentication, request handling, and platform behavior.

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
Some functionality is gated behind Cargo features to reduce compile times and dependency overhead:

- `rss`: Enables the ability to get the RSS feed data for a `webtoons.com`.
- `naver`: Enables the ability to interact with `comic.naver.com`.
- `download`: Enables the ability to download episodes, either as a single image or as multiple images.

```toml
webtoon = { version = "0.9.0", features = ["naver", "download"] }
```

## Examples

For more examples, check out the [`examples`](https://github.com/Webtoon-Studio/webtoon/tree/main/examples) folder.
