//! Module representing `webtoons.com` search.

use assumptions::Assume;

use crate::platform::webtoons::{Client, Type, Webtoon, error::ClientError};

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
        let webtoon = self;
        webtoon.id
    }

    /// Returns the [`Type`] of the Webtoon: `Original` or `Canvas`:
    #[inline]
    #[must_use]
    pub fn r#type(&self) -> Type {
        let webtoon = self;
        webtoon.r#type
    }

    /// Returns the title of the Webtoon.
    #[inline]
    #[must_use]
    pub fn title(&self) -> &str {
        let webtoon = self;
        &webtoon.title
    }

    /// Returns the thumbnail of the Webtoon.
    #[inline]
    #[must_use]
    pub fn thumbnail(&self) -> &str {
        let webtoon = self;
        &webtoon.thumbnail
    }

    /// Returns the name of the creator for the Webtoon.
    #[inline]
    #[must_use]
    pub fn creator(&self) -> &str {
        let webtoon = self;
        &webtoon.creator
    }

    /// Turns a search result into a [`Webtoon`].
    pub async fn into_webtoon(self) -> Result<Webtoon, ClientError> {
        let webtoon = self;
        let client = webtoon.client;

        let webtoon = client
            .webtoon(webtoon.id, webtoon.r#type)
            .await?
            .assumption("`webtoons.com` search should only return visible, existing series")?;

        Ok(webtoon)
    }
}
