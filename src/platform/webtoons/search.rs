use crate::{
    platform::webtoons::{Client, Type, Webtoon, error::WebtoonError},
    stdx::error::assumption,
};

/// Represents a single item in the search result.
pub struct Item {
    pub client: Client,
    pub id: u32,
    pub r#type: Type,
    pub title: String,
    pub thumbnail: String,
    pub creator: String,
}

impl Item {
    /// Returns the id of the webtoon.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the [`Type`] of the webtoon: `Original` or `Canvas`:
    #[must_use]
    pub fn r#type(&self) -> Type {
        self.r#type
    }

    /// Returns the title of the webtoon.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the thumbnail of the webtoon.
    #[must_use]
    pub fn thumbnail(&self) -> &str {
        &self.thumbnail
    }

    /// Returns the name of the creator for the webtoon.
    #[must_use]
    pub fn creator(&self) -> &str {
        &self.creator
    }

    /// Turns a search result into a [`Webtoon`] so that interaction can be done on it.
    ///
    /// Rather than having a search result in a [`Webtoon`], there is information that is not easily shared or
    /// missing from the returned API, and constructing a [`Webtoon`] each time is **very** expensive.
    ///
    /// This allows the option of quick results with only needed information while allowing one to get an interactable
    /// `Webtoon` later on.
    pub async fn into_webtoon(self) -> Result<Webtoon, WebtoonError> {
        let Some(webtoon) = self.client.webtoon(self.id, self.r#type).await? else {
            assumption!(
                "`webtoons.com` search should only return visible, existing series. Getting `None` means the webtoon is private or nonexistent, so it should never appear in search results."
            );
        };

        Ok(webtoon)
    }
}
