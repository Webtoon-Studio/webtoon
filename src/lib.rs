#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs
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

// TODO: add a callback version of `episode.posts()` so that async operations can be done on each post. This would be
// an optimization of a memory as not all posts would need to be held in memory. Can have a `webtoon.posts()` version.
// `episode.posts_for_each(|post| async { sqlx::query!("INSERT INTO posts ($1, $2, $3)").execute(&pool).await }).await?`
//
// Being generic over the error type would mean the user would be able to handle this variant alone if the situation
// calls for it.
// PostError<E: Debug> {
//     #[error("Callback function errored: {0:?}")]
//     FnError(E),
// }

// TODO: Need to expose Panel so that `&[Panel]` can be exposed so that if a frontend client wanted to have a reader
// the urls can be exposes from `Panel::url`.
