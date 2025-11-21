//! Errors that can happen when interacting with `webtoons.com`.

use crate::stdx::error::InternalInvariant;
use thiserror::Error;

mod _inner {
    use crate::stdx::error::InternalInvariant;
    use error_set::error_set;

    error_set! {
        Mud := {
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
        || DeleteError
        || ReplyError
        || BlockUserError
        || ClientError
        || SessionError

        OriginalsError := Base || ClientError

        CanvasError := Base || ClientError

        SearchError := Base || ClientError || WebtoonError

        CreatorError := {
            // TODO: Add specific languages to function docs: `Language::Zh`, `Language::De`, and `Language::Fr`
            #[display("`webtoons.com` does not support creator profiles for this language")]
            UnsupportedLanguage,
            #[display("profile page disabled by creator")]
            DisabledByCreator,
        } || Base || ClientError

        WebtoonError := Base || ClientError

        EpisodeError := {
            // TODO: missing, or deleted? I think we can figure out if an episode has at all existed.
            // Could be disabled or deleted. Could also be a draft? Have to confirm behavior.
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Base || ClientError || SessionError

        PostError := {
            // TODO: This variant might be subsumed through other errors
            #[display("insufficient permissions (not creator or poster)")]
            InvalidPermissions,
        } || Base || ClientError || SessionError

        DeleteError := {
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

        DownloadError := {
            IoError(std::io::Error),
        } || Base || ClientError

        ClientError := {
            RequestFailed(super::RequestError),
            // TODO: Some way to encode that a `RequestBuilder` failed to clone: request could not be cloned.
        }

        SessionError := {
            #[display("session not provided")]
            NotProvided,
            #[display("session invalid or expired")]
            Invalid,
        }

        Base := {
            Internal(InternalInvariant),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    OriginalsError(#[from] OriginalsError),
    #[error(transparent)]
    CanvasError(#[from] CanvasError),
    #[error(transparent)]
    SearchError(#[from] SearchError),
    #[error(transparent)]
    WebtoonError(#[from] WebtoonError),
    #[error(transparent)]
    CreatorError(#[from] CreatorError),
    #[error(transparent)]
    EpisodeError(#[from] EpisodeError),
    #[error(transparent)]
    PostError(#[from] PostError),
    #[error(transparent)]
    ReplyError(#[from] ReplyError),
    #[error(transparent)]
    PosterError(#[from] BlockUserError),

    // TODO: Should be no need to support this return.
    #[error(transparent)]
    InvalidWebtoonUrl(#[from] InvalidWebtoonUrl),

    #[cfg(feature = "download")]
    #[error(transparent)]
    DownloadError(#[from] DownloadError),
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("No session was provided")]
    NoSessionProvided,
    #[error("Provided session is invalid or expired")]
    InvalidSession,
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    InternalInvariant(#[from] InternalInvariant),
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(RequestError(err))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
#[error(transparent)]
pub struct RequestError(#[from] reqwest::Error);

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum WebtoonError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    Internal(#[from] InternalInvariant),
}

impl From<reqwest::Error> for WebtoonError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

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

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CreatorError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(
        "At this time `Language::Zh`, `Language::De`, and `Language::Fr` are not given profile pages by webtoons.com"
    )]
    UnsupportedLanguage,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
    #[error("Profile page exists, but was disabled by creator")]
    DisabledByCreator,
    #[error(transparent)]
    Internal(#[from] InternalInvariant),
}

impl From<reqwest::Error> for CreatorError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

impl From<WebtoonError> for CreatorError {
    fn from(error: WebtoonError) -> Self {
        match error {
            WebtoonError::ClientError(err) => Self::ClientError(err),
            WebtoonError::Internal(err) => Self::Internal(err),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum EpisodeError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("Episode either doesn't exist or is behind an ad or fast-pass")]
    NotViewable,
    #[error("Failed to find any panels for episode")]
    NoPanelsFound,
    #[error("Failed to find a thumbnail for episode")]
    NoThumbnailFound,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for EpisodeError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum PostError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("Not creator of webtoon or the poster")]
    InvalidPermissions,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for PostError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ReplyError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("Post is deleted, cannot post reply on deleted post")]
    DeletedPost,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for ReplyError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum BlockUserError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("Not creator of webtoon")]
    InvalidPermissions,
    #[error("Cannot block self on own webtoon")]
    BlockSelf,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for BlockUserError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum OriginalsError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
    #[error(transparent)]
    Internal(#[from] InternalInvariant),
}

impl From<reqwest::Error> for OriginalsError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CanvasError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error(transparent)]
    Internal(#[from] InternalInvariant),
}

impl From<reqwest::Error> for CanvasError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    WebtoonError(#[from] WebtoonError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for SearchError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[cfg(feature = "download")]
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[cfg(feature = "download")]
impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}
