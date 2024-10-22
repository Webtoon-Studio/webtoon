//! An abstraction for the webtoons.com website.

pub mod client;
pub mod creator;
pub mod errors;
pub mod meta;
pub mod webtoon;

pub mod canvas;
pub mod originals;

pub use meta::{Language, Type};

pub use client::{Client, ClientBuilder};

pub use creator::Creator;

pub use webtoon::Webtoon;
