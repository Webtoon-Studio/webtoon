//! An abstraction for the `comic.naver.com` website.

// TODO: Add module docs on some of domain knowledge of how Naver structures its site and how the library abstracts it where possible
//
// Naver uses a single set of ids across all three "domains" of featured, challenge, and best challenge. With this, only
// the id is needed to find a webtoon.

pub mod client;
pub mod creator;
pub mod errors;
pub mod meta;
pub mod webtoon;

pub use client::{Client, ClientBuilder};
pub use creator::Creator;
pub use meta::{Genre, Type};
pub use webtoon::Webtoon;
