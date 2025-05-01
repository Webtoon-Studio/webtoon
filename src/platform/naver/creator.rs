//! Module containing things related to a creator on webtoons.com.

use anyhow::{Context, anyhow};
use core::fmt::{self, Debug};
use parking_lot::RwLock;
use scraper::{Html, Selector};
use std::{num::ParseIntError, sync::Arc};

use super::{Client, Webtoon, errors::CreatorError};

/// Represents a creator of a `Webtoon`.
///
/// More generally this represents an account on webtoons.com.
#[derive(Clone)]
pub struct Creator {
    pub(super) client: Client,
    pub(super) username: String,
    pub(super) profile: Option<String>,
    pub(super) page: Arc<RwLock<Option<Page>>>,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for Creator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creator")
            // omitting `client`
            .field("username", &self.username)
            .field("profile", &self.profile)
            .finish()
    }
}

#[derive(Debug)]
pub(super) struct Page {
    pub username: String,
    pub followers: u32,
    pub id: String,
}

impl Creator {
    /// Returns a `Creators` username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a `Creators` profile segment in `https://comic.naver.com/community/u/{profile}`
    ///
    /// Not all creators for a story have a webtoons profile.
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns a `Creators` id.
    ///
    /// Sometimes this is just the `profile` but with the `_` prefix stripped.
    /// If creator has no webtoon profile then this will always return `None`.
    ///
    /// # Errors
    ///
    /// Will error if failed to scrape the creators profile page.
    pub async fn id(&self) -> Result<Option<String>, CreatorError> {
        if let Some(page) = &*self.page.read() {
            Ok(Some(page.id.clone()))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let page = page(profile, &self.client).await?;
            let followers = page.as_ref().map(|page| page.id.clone());
            *self.page.write() = page;
            Ok(followers)
        }
    }

    /// Returns the number of followers for the `Creator`.
    pub async fn followers(&self) -> Result<Option<u32>, CreatorError> {
        if let Some(page) = &*self.page.read() {
            Ok(Some(page.followers))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let page = page(profile, &self.client).await?;
            let followers = page.as_ref().map(|page| page.followers);
            *self.page.write() = page;
            Ok(followers)
        }
    }

    /// Scrapes the profile page for the public facing webtoons.
    ///
    /// # Returns
    ///
    /// Will return `Some` if there is a Webtoon profile, otherwise it will return `None`.
    /// This is for creators where there are no profile, either due to being a Korean based creator,
    /// or the language version of webtoons.com does not support profile pages.
    ///
    /// If there are no viewable webtoons, it will return an empty `Vec`.
    ///
    /// # Errors
    ///
    /// Will error if scrape encountered an unexpected html shape, or if network request encounter issues.
    pub async fn webtoons(&self) -> Result<Option<Vec<Webtoon>>, CreatorError> {
        let Some(profile) = self
            .profile
            .as_deref()
            // Profiles can be prefixed with `_` but the url needs it trimmed to work.
            .map(|profile| profile.trim_start_matches('_'))
        else {
            return Ok(None);
        };

        let response = match self.client.get_webtoons_from_creator_page(profile).await {
            Ok(response) => response,
            Err(_) => {
                let page = page(profile, &self.client).await?;
                let profile = page
                    .as_ref()
                    .map(|page| page.id.clone())
                    .context("failed to find creator profile property on creator page html")?;
                *self.page.write() = page;

                self.client.get_webtoons_from_creator_page(&profile).await?
            }
        };

        let json: api::Root = serde_json::from_str(&response.text().await?)
            .map_err(|err| CreatorError::Unexpected(err.into()))?;

        let mut webtoons = Vec::with_capacity(json.result.series.len());

        for webtoon in json.result.series {
            let id = webtoon
                .id
                .parse::<u32>()
                .context("failed to parse webtoon id to number")?;

            webtoons.push(Webtoon::new_with_client(id, &self.client).await?);
        }

        Ok(Some(webtoons))
    }

    /// Clears the cached metadata for the current `Creator`, forcing future requests to retrieve fresh data from the network.
    ///
    /// ### Behavior
    ///
    /// - **Cache Eviction**:
    ///   - This method clears the cached creator metadata (such as username, followers, and other page information) that has been stored for performance reasons.
    ///   - After calling this method, subsequent calls that rely on this metadata will trigger a network request to re-fetch the data.
    ///
    /// ### Use Case
    ///
    /// - Use this method if you suspect the cached data is outdated or if you want to ensure that future data retrieval reflects the latest updates from the creator's page.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(creator) = client.creator("JennyToons", Language::En).await? {
    /// creator.evict_cache().await;
    /// println!("Cache cleared. Future requests will fetch fresh data.");
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Notes
    ///
    /// - There are no errors returned from this function, as it only resets the cache.
    /// - Cache eviction is useful if the creators metadata has changed or if up-to-date information is needed for further operations.
    pub fn evict_cache(&self) {
        *self.page.write() = None;
    }
}

pub(super) async fn page(profile: &str, client: &Client) -> Result<Option<Page>, CreatorError> {
    let response = client.get_creator_page(profile).await?;

    if response.status() == 404 {
        return Ok(None);
    }

    if response.status() == 400 {
        return Err(CreatorError::DisabledByCreator);
    }

    let document = response.text().await?;

    let html = Html::parse_document(&document);

    Ok(Some(Page {
        username: username(&html)?,
        followers: followers(&html)?,
        id: id(&html)?,
    }))
}

fn username(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse(r#"head>meta[name="author"]"#) //
        .expect(r#"`head>meta[name="author"]` should be a valid selector"#);

    if let Some(element) = html.select(&selector).next() {
        if let Some(text) = element.value().attr("content") {
            eprintln!("{text}");
            return Ok(text.to_string());
        }
    }

    Err(CreatorError::Unexpected(anyhow!(
        "failed to find creator username on creator page"
    )))
}

fn followers(html: &Html) -> Result<u32, CreatorError> {
    let selector = Selector::parse("span.x7o99n6") //
        .expect("`span.x7o99n6` should be a valid selector");

    if let Some(element) = html.select(&selector).next() {
        if let Some(text) = element.text().nth(1) {
            eprintln!("{text}");
            return text
                .replace(',', "")
                .parse()
                .map_err(|err: ParseIntError| CreatorError::Unexpected(err.into()));
        }
    }

    Err(CreatorError::Unexpected(anyhow!(
        "failed to find creator follower count on creator page"
    )))
}

fn id(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse("script").expect("`script` should be a valid selector");

    for element in html.select(&selector) {
        if let Some(inner) = element.text().next() {
            if let Some(idx) = inner.find("creatorId") {
                let mut quotes = 0;

                // EXAMPLE: `creatorId\":\"n5z4d\"`
                let bytes = &inner.as_bytes()[idx..];

                let mut start = 0;
                let mut idx = 0;

                let mut found_start = false;

                loop {
                    if bytes[idx] == b'"' {
                        quotes += 1;
                    }

                    if quotes == 2 && !found_start {
                        // `creatorId\":\"n5z4d\"`
                        //           idx ^
                        // Advance beyond quote:
                        //
                        // `creatorId\":\"n5z4d\"`
                        //          start ^
                        start = idx + 1;
                        found_start = true;
                    }

                    if quotes == 3 {
                        // `creatorId\":\"n5z4d\"`
                        //          start ^     ^ idx
                        return Ok(std::str::from_utf8(&bytes[start..idx])
                            .expect("parsed creator id should be valid utf-8")
                            .trim_end_matches('\\')
                            .to_string());
                    }

                    idx += 1;
                }
            }
        }
    }

    Err(CreatorError::Unexpected(anyhow!(
        "failed to find alternate creator profile in creatior page html"
    )))
}

#[allow(unused)]
mod api {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Root {
        pub result: Result1,
        pub status: String,
    }
    #[derive(Deserialize)]
    pub struct Result1 {
        pub series: Vec<Series>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Series {
        pub id: String,
    }
}
