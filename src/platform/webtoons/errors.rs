//! Errors that can happen when interacting with webtoons.com.

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
    #[error(transparent)]
    #[cfg(feature = "download")]
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
    #[error("Rate limit was exceeded")]
    RateLimitExceeded(u64),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Unexpected(anyhow::Error::from(error))
    }
}

#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum WebtoonError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("{0}")]
    InvalidUrl(&'static str),
    #[error("No genre was found for webtoon")]
    NoGenre,
    #[error(transparent)]
    MalformedUrl(#[from] url::ParseError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for WebtoonError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
    }
}

#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CreatorError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("At this time `Language::Zh`, `Language::De`, and `Language::Fr` are not given profile pages by webtoons.com")]
    UnsupportedLanguage,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for CreatorError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
}

impl From<reqwest::Error> for OriginalsError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
}

impl From<reqwest::Error> for CanvasError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::from(error))
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
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::from(error))
    }
}
