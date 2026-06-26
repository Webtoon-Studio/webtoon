//! Errors that can happen when interacting with `webtoons.com`.
#![allow(missing_docs)]

use thiserror::Error;

#[cfg(feature = "download")]
pub use _inner::SavePanelError;

pub use _inner::{
    CanvasError, ClientBuilderError, ClientError, CreatorError, CreatorWebtoonsError, EpisodeError,
    Error, OriginalsError, RssError, SearchError, SessionError, UserInfoError,
    WebtoonEpisodesError, WebtoonError, WebtoonLikesError, WebtoonPostsError,
    WebtoonSubscribersError, WebtoonViewsError,
};

#[derive(Debug, Error)]
#[error(transparent)]
pub struct RequestError(#[from] pub(crate) reqwest::Error);

// TODO: Create a `Url` in `webtoon::homepage` that validates the expected structure, i.e. "Parse, don't validate"
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
    use crate::{platform::webtoons::webtoon::post::id::ParsePostIdError, stdx::error::Assumption};
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
            ParseIdError(ParsePostIdError),
        }
        || Base
        || OriginalsError
        || CanvasError
        || SearchError
        || CreatorError
        || WebtoonError
        || EpisodeError
        || RequestError
        || SessionError
        || WebtoonPostsError
        || ClientError

        OriginalsError := Base || RequestError

        CanvasError := {
            #[display("range `start` cannot be lower than `end`")]
            InvalidRange,
        } || Base || RequestError

        SearchError := Base || RequestError

        CreatorWebtoonsError := CreatorError || ClientError

        CreatorError := {
            #[display("invalid creator profile")]
            InvalidCreatorProfile,
        } || Base || RequestError

        WebtoonError := Base || RequestError

        RssError :=  Base || RequestError

        EpisodeError := {
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Base || RequestError

        WebtoonPostsError := Base || RequestError || InvalidSession

        WebtoonLikesError := Base || RequestError

        // TODO: Need to add `InvalidPermissions` as session provided might be a
        // valid one, but not the one needed for the specific webtoon.
        WebtoonEpisodesError :=  Base || RequestError || InvalidSession
        WebtoonViewsError := Base || RequestError
        WebtoonSubscribersError := Base || RequestError

        SessionError :=  Base || RequestError || NoSessionProvided || InvalidSession

        UserInfoError := Base || RequestError

        ClientBuilderError := {
            BuildFailed,
        }

        ClientError := {
            #[display("only the english `webtoons.com` is supported")]
            UnsupportedLanguage
        } || Base || RequestError

        SavePanelError := {
            IoError(std::io::Error),
        } || Base || RequestError

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

        RequestError := {
            RequestFailed(super::RequestError),
        }

        Base := {
            Internal(Assumption),
        }
    }

    impl From<SessionError> for WebtoonViewsError {
        #[track_caller]
        fn from(err: SessionError) -> Self {
            match err {
                SessionError::RequestFailed(request_error) => Self::RequestFailed(request_error),
                SessionError::Internal(assumption) => Self::Internal(assumption),
                SessionError::NoSessionProvided => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is NoSessionProvided"
                ),
                SessionError::InvalidSession => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is InvalidSession"
                ),
            }
        }
    }

    impl From<SessionError> for WebtoonSubscribersError {
        #[track_caller]
        fn from(err: SessionError) -> Self {
            match err {
                SessionError::RequestFailed(request_error) => Self::RequestFailed(request_error),
                SessionError::Internal(assumption) => Self::Internal(assumption),
                SessionError::NoSessionProvided => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is NoSessionProvided"
                ),
                SessionError::InvalidSession => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is InvalidSession"
                ),
            }
        }
    }

    impl From<SessionError> for WebtoonEpisodesError {
        #[track_caller]
        fn from(err: SessionError) -> Self {
            match err {
                SessionError::RequestFailed(request_error) => Self::RequestFailed(request_error),
                SessionError::Internal(assumption) => Self::Internal(assumption),
                SessionError::NoSessionProvided => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is NoSessionProvided"
                ),
                SessionError::InvalidSession => unreachable!(
                    "should have a guard before any `?` for `from` that checks if SessionError is InvalidSession"
                ),
            }
        }
    }
}
