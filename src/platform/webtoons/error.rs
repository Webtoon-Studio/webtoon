//! Errors that can happen when interacting with `webtoons.com`.
#![allow(missing_docs)]

use thiserror::Error;

#[cfg(feature = "download")]
pub use _inner::DownloadError;

pub use _inner::{
    CanvasError, ClientBuilderError, CreatorError, EpisodeError, EpisodesError, Error, LikesError,
    OriginalsError, PostsError, RssError, SearchError, SessionError, SubscribersError,
    UserInfoError, ViewsError, WebtoonError,
};

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

mod _inner {
    use crate::{platform::webtoons::webtoon::post::id::ParseIdError, stdx::error::Assumption};
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
            ParseIdError(ParseIdError),
        }
        || Base
        || OriginalsError
        || CanvasError
        || SearchError
        || CreatorError
        || WebtoonError
        || EpisodeError
        || ClientError
        || SessionError
        || PostsError

        OriginalsError := Base || ClientError

        CanvasError := {
            #[display("range `start` cannot be lower than `end`")]
            InvalidRange,
        } || Base || ClientError

        SearchError := Base || ClientError

        CreatorError := {
            #[display("`webtoons.com` does not support creator profiles for this language")]
            UnsupportedLanguage,
            #[display("invalid creator profile")]
            InvalidCreatorProfile,
        } || Base || ClientError

        WebtoonError := Base || ClientError

        RssError :=  Base || ClientError

        EpisodeError := {
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Base || ClientError

        PostsError := Base || ClientError || InvalidSession

        LikesError := Base || ClientError

        // TODO: Need to add `InvalidPermissions` as session provided might be a
        // valid one, but not the one needed for the specific webtoon.
        EpisodesError :=  Base || ClientError || InvalidSession
        ViewsError := Base || ClientError || InvalidSession
        SubscribersError := Base || ClientError || InvalidSession

        SessionError :=  Base || ClientError || NoSessionProvided || InvalidSession

        UserInfoError := Base || ClientError

        ClientBuilderError := {
            BuildFailed,
        }

        DownloadError := {
            IoError(std::io::Error),
        } || Base || ClientError

        // --- Internal ---

        InvalidSession := {
            #[display("session invalid or expired")]
            InvalidSession,
        }

        NoSessionProvided := {
            #[display("session not provided")]
            NoSessionProvided,
        }

        InvalidPermissions := {
            #[display("insufficient permissions (not creator)")]
            InvalidPermissions,
        }

        ClientError := {
            RequestFailed(super::RequestError),
        }

        Base := {
            Internal(Assumption),
        }
    }
}
