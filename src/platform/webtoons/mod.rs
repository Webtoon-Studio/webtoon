//! An abstraction for the `webtoons.com` website.

mod dashboard;
mod search;

pub mod canvas;
pub mod creator;
pub mod meta;
pub mod originals;

pub mod client;
pub mod error;
pub mod webtoon;

pub use client::{Client, ClientBuilder};
pub use creator::Creator;
pub use meta::{Language, Type};
pub use webtoon::Webtoon;
