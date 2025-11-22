//! Errors that can happen when interacting with `webtoons.com`.
#![allow(missing_docs)]

use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
#[error(transparent)]
pub struct RequestError(#[from] pub(crate) reqwest::Error);

/// Represents an invalid `webtoons.com` Webtoon homepage URL.
///
/// Given how exact the format is, and the unlikely nature of something actionable
/// being done, this error is merely a message carrier that says what expectations
/// were violated.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct InvalidWebtoonUrl(String);

impl InvalidWebtoonUrl {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[cfg(feature = "download")]
pub use _inner::DownloadError;

pub(crate) use _inner::{
    ApiTokenError, CreatorWebtoonsError, ReactTokenError, StatsDashboardError, UserInfoError,
};
pub use _inner::{
    BlockUserError, CanvasError, ClientBuilderError, ClientError, CreatorError, DeletePostError,
    EpisodeError, Error, LikesError, OriginalsError, PostError, PostsError, ReplyError,
    SearchError, SessionError, WebtoonError,
};

mod _inner {
    use crate::{
        platform::webtoons::webtoon::post::id::ParseIdError, stdx::error::InternalInvariant,
    };
    use error_set::error_set;

    error_set! {
        #[expect(
            clippy::error_impl_error,
            reason = "`Error` is a ball of mud enum thats built through codegen; only meant for prototyping"
        )]
        Error := {
            InvalidWebtoonUrl(super::InvalidWebtoonUrl),
            #[cfg(feature = "download")]
            IoError(std::io::Error),
        }
        || Base
        || OriginalsError
        || CanvasError
        || SearchError
        || CreatorError
        || WebtoonError
        || EpisodeError
        || PostError
        || DeletePostError
        || ReplyError
        || BlockUserError
        || ClientError
        || SessionError
        || PostsError

        OriginalsError := Base || ClientError

        CanvasError := Base || ClientError

        SearchError := Base || ClientError || WebtoonError

        CreatorError := {
            // TODO: Add specific languages to function docs: `Language::Zh`, `Language::De`, and `Language::Fr`
            #[display("`webtoons.com` does not support creator profiles for this language")]
            UnsupportedLanguage,
            #[display("profile page disabled by creator")]
            DisabledByCreator,
        } || Base || ClientError || WebtoonError

        WebtoonError := Base || ClientError || StatsDashboardError

        StatsDashboardError := Base || ClientError || SessionError

        EpisodeError := {
            // TODO: missing, or deleted? I think we can figure out if an episode has at all existed.
            // Could be disabled or deleted. Could also be a draft? Have to confirm behavior.
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Base || ClientError || SessionError

        PostsError := {
            // TODO See if this is needed
            ParseIdError(ParseIdError)
        } || Base || ClientError || SessionError

        LikesError :=  Base || ClientError || SessionError

        PostError := {
            // TODO: This variant might be subsumed through other errors
            #[display("insufficient permissions (not creator or poster)")]
            InvalidPermissions,
        } || Base || ClientError || SessionError

        DeletePostError := {
            #[display("insufficient permissions (not creator or poster)")]
            InvalidPermissions,
        } || Base || ClientError || SessionError

        ReplyError := {
            #[display("cannot reply to a deleted post")]
            DeletedPost,
        } || Base || ClientError || SessionError

        BlockUserError := {
            #[display("cannot block self on own webtoon")]
            BlockSelf,
            #[display("insufficient permissions (not creator)")]
            NotCreator,
        } || Base || ClientError || SessionError

        UserInfoError := Base || ClientError || SessionError
        ReactTokenError := Base || ClientError || SessionError
        ApiTokenError := Base || ClientError || SessionError
        CreatorWebtoonsError := Base || ClientError

        DownloadError := {
            IoError(std::io::Error),
        } || Base || ClientError

        SessionError := {
            #[display("session not provided")]
            NoSessionProvided,
            #[display("session invalid or expired")]
            InvalidSession,
        } || Base || ClientError

        ClientError := {
            RequestFailed(super::RequestError),
            // TODO: Some way to encode that a `RequestBuilder` failed to clone: request could not be cloned.
        }

        ClientBuilderError := {
            BuildFailed,
        }

        Base := {
            Internal(InternalInvariant),
        }
    }
}
