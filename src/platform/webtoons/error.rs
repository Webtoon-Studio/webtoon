//! Errors that can happen when interacting with `webtoons.com`.
#![allow(missing_docs)]

#[cfg(feature = "download")]
pub use _inner::SavePanelError;

pub use _inner::{
    CanvasError, ClientBuilderError, ClientError, CreatorError, CreatorWebtoonsError, EpisodeError,
    EpisodesError, Error, InvalidWebtoonUrl, LikesError, OriginalsError, PostsError, SearchError,
    SessionError, SubscribersError, UserInfoError, ViewsError, WebtoonError,
};

mod _inner {
    use assumptions::Assumption;
    use error_set::error_set;

    error_set! {
        /// Catch-all error for prototyping. Prefer specific types in production code.
        #[expect(clippy::error_impl_error, reason = "catch-all for prototyping only")]
        Error := {
            InvalidWebtoonUrl(super::InvalidWebtoonUrl),
            #[cfg(feature = "download")]
            IoError(std::io::Error),
        }
        || OriginalsError
        || CanvasError
        || SearchError
        || CreatorError
        || WebtoonError
        || EpisodeError
        || SessionError
        || PostsError
        || ClientError
        || Network
        || Internal

        SearchError := Internal || Network

        OriginalsError := Internal || Network

        CanvasError := Internal || Network

        // TODO: See if we need the ClientError::UnsupportedLanguage
        CreatorWebtoonsError := CreatorError || ClientError

        CreatorError := {
            #[display("invalid creator profile")]
            InvalidCreatorProfile,
        } || Internal || Network

        WebtoonError := Internal || Network

        EpisodeError := {
            #[display("episode not viewable (missing, ad-locked, or fast-pass)")]
            NotViewable,
        } || Internal || Network

        PostsError := Internal || Network || InvalidSession

        LikesError := Internal || Network

        EpisodesError :=  Internal || Network

        ViewsError := Internal || Network

        SubscribersError := Internal || Network

        SessionError :=  Internal || Network || NoSessionProvided || InvalidSession

        UserInfoError := Internal || Network

        /// Error saving downloaded panels to disk.
        SavePanelError := {
            IoError(std::io::Error),
        } || Internal || Network

        /// Error building a [`Client`](crate::platform::webtoons::client::Client).
        ClientBuilderError := {
            #[display("failed to build the HTTP client (TLS or DNS initialization failed)")]
            BuildFailed,
        }

        ClientError := {
            #[display("only the english `webtoons.com` is supported")]
            UnsupportedLanguage
        } || Internal || Network

        /// Represents an invalid `webtoons.com` Webtoon homepage URL.
        ///
        /// Given how exact the format is, and the unlikely nature of something actionable
        /// being done, this error is merely a message carrier that says what expectations
        /// were violated.
        InvalidWebtoonUrl := {
            /// Unsupported language.
            #[display("only the english `webtoons.com` is supported")]
            UnsupportedLanguage,
            /// Url had an unexpected layout.
            #[display("`{url}` was malformed: {reason}")]
            Malformed{
                url: String,
                reason: String,
            },
        }

        // ---------------------------------------------------------------------

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

        // ---------------------------------------------------------------------

        Network := {
            #[display("{0}")]
            RequestFailed(reqwest::Error),
        }

        Internal := {
            Internal(Assumption),
        }
    }

    impl From<SessionError> for ViewsError {
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

    impl From<SessionError> for SubscribersError {
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

    impl From<SessionError> for EpisodesError {
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
