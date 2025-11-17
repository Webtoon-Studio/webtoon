//! Errors that can happen when interacting with `comic.naver.com`.

use thiserror::Error;

#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    WebtoonError(#[from] WebtoonError),
    #[error(transparent)]
    CreatorError(#[from] CreatorError),
    #[error(transparent)]
    EpisodeError(#[from] EpisodeError),
    #[error(transparent)]
    PostError(#[from] PostError),

    #[cfg(feature = "download")]
    #[error(transparent)]
    DownloadError(#[from] DownloadError),
}

#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ClientError {
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
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
    #[error("Profile page exists, but was disabled by creator")]
    DisabledByCreator,
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
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<reqwest::Error> for PostError {
    fn from(error: reqwest::Error) -> Self {
        Self::ClientError(ClientError::Unexpected(anyhow::Error::from(error)))
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
