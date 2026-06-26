//! Module representing `webtoons.com` search.

use crate::{
    platform::webtoons::{Client, Type, Webtoon, error::ClientError},
    stdx::error::Assume,
};

/// Represents a single item in the search result.
pub struct Item {
    pub(crate) client: Client,
    pub(crate) id: u32,
    pub(crate) r#type: Type,
    pub(crate) title: String,
    pub(crate) thumbnail: String,
    pub(crate) creator: String,
}

impl Item {
    /// Returns the id of the Webtoon.
    #[inline]
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the [`Type`] of the Webtoon: `Original` or `Canvas`:
    #[inline]
    #[must_use]
    pub fn r#type(&self) -> Type {
        self.r#type
    }

    /// Returns the title of the Webtoon.
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the thumbnail of the Webtoon.
    #[inline]
    #[must_use]
    pub fn thumbnail(&self) -> &str {
        &self.thumbnail
    }

    /// Returns the name of the creator for the Webtoon.
    #[inline]
    #[must_use]
    pub fn creator(&self) -> &str {
        &self.creator
    }

    /// Turns a search result into a [`Webtoon`].
    pub async fn into_webtoon(self) -> Result<Webtoon, ClientError> {
        let webtoon = self
            .client
            .webtoon(self.id, self.r#type)
            .await?
            .assumption("`webtoons.com` search should only return visible, existing series.")?;

        Ok(webtoon)
    }
}
