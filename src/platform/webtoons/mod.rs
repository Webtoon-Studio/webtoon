//! An abstraction for the `webtoons.com` website.

mod dashboard;

pub mod canvas;
pub mod client;
pub mod creator;
pub mod error;
pub mod meta;
pub mod originals;
pub mod search;
pub mod webtoon;

pub use client::{Client, ClientBuilder};
pub use creator::Creator;
pub use meta::{Language, Type};
pub use webtoon::Webtoon;
