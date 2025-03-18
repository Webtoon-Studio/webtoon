//! Module for webtoons.com search API.

use serde::{Deserialize, Serialize};

use crate::platform::webtoons::{Type, Webtoon, errors::WebtoonError};

use super::Client;

/// Represents a single item in the search result.
pub struct Item {
    pub(super) client: Client,
    pub(super) id: u32,
    pub(super) r#type: Type,
    pub(super) title: String,
    pub(super) thumbnail: String,
    pub(super) creator: String,
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
        let webtoon =
            self.client.webtoon(self.id, self.r#type).await?.expect(
                "Webtoon info came directly from webtoons.com so should be a valid webtoon",
            );
        Ok(webtoon)
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Api {
    pub result: SearchResult,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SearchResult {
    pub challenge_title_list: Option<Canvas>,
    pub webtoon_title_list: Option<Originals>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Originals {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Canvas {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Data {
    pub content_id: String,
    pub content_sub_type: String,
    pub extra: Extra,
    pub name: String,
    pub service_type: String,
    pub thumbnail: Thumbnail,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Illustrator {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Writer {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Thumbnail {
    pub domain: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Pagination {
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Extra {
    pub illustrator: Illustrator,
    pub unsuitable_for_children: Option<bool>,
    pub writer: Writer,
}
