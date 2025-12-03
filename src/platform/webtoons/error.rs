//! Errors that can happen when interacting with `webtoons.com`.
#![allow(missing_docs)]

use thiserror::Error;

#[cfg(feature = "download")]
pub use _inner::DownloadError;

// pub(crate) use _inner::RssError;
pub use _inner::{
    BlockUserError, CanvasError, ClientBuilderError, ClientError, CreatorError, DeletePostError,
    EpisodeError, EpisodesError, Error, LikesError, OriginalsError, PostError, PostsError,
    ReplyError, RssError, SearchError, SessionError, SubscribersError, UserInfoError, ViewsError,
    WebtoonError,
};

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

mod _inner {
    use crate::stdx::error::Assumption;
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

        CanvasError := {
            #[display("range `start` cannot be lower than `end`")]
            InvalidRange,
        } || Base || ClientError

        SearchError := Base || ClientError

        CreatorError := {
            #[display("`webtoons.com` does not support creator profiles for this language")]
            UnsupportedLanguage,
            #[display("profile page disabled by creator")]
            DisabledByCreator,
        } || Base || ClientError

        WebtoonError := Base || ClientError

        EpisodeError := {
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Base || ClientError

        PostsError := {
            #[display("session invalid or expired")]
            InvalidSession
        } || Base || ClientError

        LikesError := {
            #[display("session invalid or expired")]
            InvalidSession
        } || Base || ClientError

        ViewsError := {
            #[display("session invalid or expired")]
            InvalidSession
        } || Base || ClientError

        SubscribersError := {
            #[display("session invalid or expired")]
            InvalidSession
        } || Base || ClientError

        EpisodesError := {
            #[display("session invalid or expired")]
            InvalidSession
        } || Base || ClientError

        RssError :=  Base || ClientError

        PostError := {
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

        DownloadError := {
            IoError(std::io::Error),
        } || Base || ClientError

        SessionError := {
            #[display("session not provided")]
            NoSessionProvided,
            #[display("session invalid or expired")]
            InvalidSession,
        } || Base || ClientError

        UserInfoError := Base || ClientError

        ClientError := {
            RequestFailed(super::RequestError),
        }

        ClientBuilderError := {
            BuildFailed,
        }

        Base := {
            Internal(Assumption),
        }
    }
}
