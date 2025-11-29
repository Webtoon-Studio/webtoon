#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rust_2018_idioms,
    // clippy::unwrap_used,
    // clippy::expect_used,
    clippy::panic,
    clippy::map_err_ignore,
    clippy::missing_panics_doc,
    clippy::match_wildcard_for_single_variants,
    clippy::wildcard_in_or_patterns,
    clippy::await_holding_lock,
    // clippy::implicit_clone,
    clippy::default_trait_access,
    clippy::let_underscore_future,
    // clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::manual_range_contains,
    // clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::ptr_as_ptr,
    clippy::get_first,
    clippy::manual_split_once,
    clippy::manual_is_ascii_check,
    clippy::manual_map,
    clippy::manual_async_fn,
    clippy::mutex_integer,
    clippy::needless_pass_by_value,
    clippy::needless_option_as_deref,
    clippy::result_large_err,
    // clippy::error_impl_error,
    clippy::useless_let_if_seq,
    clippy::match_like_matches_macro,
    clippy::manual_non_exhaustive,
    // clippy::todo, // TODO: Need to implement `webtoons` `homepage/*`
    clippy::unimplemented,
    clippy::manual_ok_or,
    clippy::manual_unwrap_or,
    // clippy::indexing_slicing
)]
#![allow(
    clippy::option_if_let_else,
    clippy::missing_const_for_fn,
    clippy::cargo_common_metadata,
    clippy::multiple_crate_versions,
    clippy::missing_errors_doc,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_closure
)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
mod stdx;

pub mod platform;

mod private {
    pub trait Sealed {}

    impl Sealed for u32 {}
}
