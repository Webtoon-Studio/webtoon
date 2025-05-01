//! An abstraction for the `comic.naver.com` website.

pub mod client;
pub mod creator;
pub mod errors;
pub mod meta;
pub mod webtoon;

pub use client::{Client, ClientBuilder};
pub use creator::Creator;
pub use meta::Type;
pub use webtoon::Webtoon;
