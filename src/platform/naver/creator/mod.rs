//! Module containing things related to a creator on webtoons.com.

use anyhow::Context;
use core::fmt::{self, Debug};

use crate::platform::naver;

use super::{errors::CreatorError, Client, Webtoon};

// TODO: Implement page caching

/// Represents a creator of a webtoon.
///
/// More generally this represents an account on webtoons.com.
#[derive(Clone)]
pub struct Creator {
    pub(super) client: Client,
    pub(super) username: String,
    pub(super) id: u64,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for Creator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creator")
            // Omitting `client`
            .field("username", &self.username)
            .field("id", &self.id)
            .finish()
    }
}

impl Creator {
    /// Returns a `Creators` username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Scrapes the profile page for the public facing webtoons.
    ///
    /// # Returns
    ///
    /// Will return `Some` if there is a Webtoon profile, otherwise it will return `None`.
    /// This is for creators where there are no profile, either due to being a Korean based creator,
    /// or the language version of webtoons.com does not support profile pages.
    ///
    /// **Unsupported Languages**: Korean, Chinese, French, and German.
    ///
    /// If there are no viewable webtoons, it will return an empty `Vec`.
    ///
    /// # Errors
    ///
    /// Will error if scrape encountered an unexpected html shape, or if network request encounter issues.
    pub async fn webtoons(&self) -> Result<Option<Vec<Webtoon>>, CreatorError> {
        todo!()
    }
}

impl From<(naver::client::info::Creator, &Client)> for Creator {
    fn from((creator, client): (naver::client::info::Creator, &Client)) -> Self {
        Self {
            client: client.clone(),
            username: creator.username,
            id: creator.id,
        }
    }
}
