//! Errors that can happen when interacting with `webtoons.com`.

use thiserror::Error;

#[allow(missing_docs)]
#[non_exhaustive]
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
    PosterError(#[from] PosterError),

    // TODO: Should be no need to support this return.
    #[error(transparent)]
    InvalidWebtoonUrl(#[from] InvalidWebtoonUrl),

    #[cfg(feature = "download")]
    #[error(transparent)]
    DownloadError(#[from] DownloadError),
}

#[allow(missing_docs)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PosterError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("Not creator of webtoon")]
    InvalidPermissions,
    #[error("Cannot block self on own webtoon")]
    BlockSelf,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for PosterError {
    fn from(err: reqwest::Error) -> Self {
        Self::ClientError(ClientError::RequestError(RequestError(err)))
    }
}

#[allow(missing_docs)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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

/// Represents internal invariants that were violated.
///
/// If this is returned, this is considered a bug that must be fixed!
///
/// This error is not actionable by library user, and must be fixed via internal
/// code changes! Please open an issue!
///
/// # Use
///
/// The rule of thumb for this error is that it is only used when interacting
/// with the platform, as opposed to input data that might be passed to the library.
///
/// In an effort to maintain correctness in the library, liberal use of pre and
/// post checks are used to make sure any changes that happen underneath the
/// library are caught and fixed as soon as possible.
#[derive(Debug, Error)]
#[error("internal invariant violated: {0}")]
pub struct InternalInvariant(String);

impl From<String> for InternalInvariant {
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

macro_rules! invariant {
    ($msg:literal $(, $args:expr)* ) => {{
        return Err($crate::platform::webtoons::errors::InternalInvariant::from( format!($msg $(, $args)*)).into());
    }};
    ($err:expr) => {{
        return Err($crate::platform::webtoons::errors::InternalInvariant::from( format!("{}", $err)).into());
    }};
    ($cond:expr, $msg:literal $(, $args:expr)* ) => {{
        if !$cond {
        return Err($crate::platform::webtoons::errors::InternalInvariant::from( format!("`{}`, {}", stringify!($cond), format!($msg $(, $args)*))).into());
        }
    }};
}

pub(crate) use invariant;

pub(crate) trait Invariant<T> {
    type Output;

    fn invariant(self, msg: impl Into<String>) -> Self::Output;
    // fn with_invariant(self, msg: impl FnOnce() -> String) -> Self::Output;
}

impl<T> Invariant<T> for Option<T> {
    type Output = Result<T, InternalInvariant>;

    fn invariant(self, msg: impl Into<String>) -> Self::Output {
        self.ok_or_else(|| InternalInvariant(msg.into()))
    }

    // fn with_invariant(self, msg: impl FnOnce() -> String) -> Self::Output {
    //     self.ok_or_else(|| InternalInvariant(msg()))
    // }
}

impl<T, E> Invariant<T> for Result<T, E> {
    type Output = Result<T, InternalInvariant>;

    fn invariant(self, msg: impl Into<String>) -> Self::Output {
        self.map_err(|_| InternalInvariant(msg.into()))
    }

    // fn with_invariant(self, msg: impl FnOnce() -> String) -> Self::Output {
    //     self.map_err(|_| InternalInvariant(msg()))
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_with_message() -> Result<(), InternalInvariant> {
        invariant!("failed to uphold assumption");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_bail_on_condition_fail_with_message() -> Result<(), InternalInvariant> {
        let panels: Vec<()> = vec![];
        invariant!(
            !panels.is_empty(),
            "episode panels should not be empty, but was {}",
            panels.len()
        );
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_have_format_args() -> Result<(), InternalInvariant> {
        let arg = "foo";
        invariant!("failed to uphold assumption with {arg} and {}", "bar");
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_invariant() -> Result<(), InternalInvariant> {
        let err: Option<()> = None;
        err.invariant("failed to find `a.img` html tag on webtoon homepage")?;
        Ok(())
    }

    #[test]
    #[ignore = "this should only be manually verified"]
    fn should_error_with_internal_invariant2() -> Result<(), InternalInvariant> {
        let webtoon = "Test";
        let err: Option<()> = None;
        err.invariant(format!(
            "failed to find `a.img` html tag on webtoon homepage for `{webtoon}`"
        ))?;
        Ok(())
    }
}
