#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rust_2018_idioms
)]
#![allow(
    clippy::option_if_let_else,
    clippy::missing_const_for_fn,
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions
)]
#![doc = include_str!("../README.md")]
mod stdx;

pub mod platform;

mod private {
    pub trait Sealed {}
}
