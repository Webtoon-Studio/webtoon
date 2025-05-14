//! Module containing things related to a creator on `comic.naver.com`.

use anyhow::{Context, anyhow};
use core::fmt::{self, Debug};
use parking_lot::RwLock;
use scraper::{Html, Selector};
use std::{num::ParseIntError, sync::Arc};

use super::{Client, Webtoon, errors::CreatorError};

/// Represents a creator of a `Webtoon`.
///
/// More generally, this represents an account on `comic.naver.com`.
///
/// This type is not constructed directly, instead it is gotten through a [`Client`] via [`Client::creator()`].
///
/// # Example
///
/// ```
/// # use webtoon::platform::naver::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(creator) = client.creator("_n41b8i").await? else {
///     unreachable!("profile is known to exist");
/// };
///
/// assert_eq!("호리", creator.username());
/// # Ok(())
/// # }
/// ```
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
    /// Returns a Creators username.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("_wig53").await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!("홍대의", creator.username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a Creators profile.
    ///
    /// This corresponds to the segment in: `https://comic.naver.com/community/u/{profile}`
    ///
    /// Most often this is just the profile that was passed to [`Client::creator()`].
    ///
    /// # Caveats
    ///
    /// Not all creators for a story have a profile. In such case, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("_0jhat").await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some("_0jhat"), creator.profile());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns a Creators id.
    ///
    /// Sometimes this is just the [`profile()`](Creator::profile()), but with the `_` prefix stripped.
    ///
    /// If creator has no Webtoon profile, then this will always return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("_7box2").await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some("7box2"), creator.id().await?.as_deref());
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the number of followers for the Creator.
    ///
    /// More specifically this corresponds to the number of `관심`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("2jiho").await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// println!("`{}` has `{:?}` followers", creator.username(), creator.followers().await?);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns a list of any public facing webtoons the creator is involved with.
    ///
    /// # Returns
    ///
    /// Will return `Some` if there is a Webtoon profile, otherwise it will return `None`. If there are no viewable
    /// Webtoon's, it will return an empty `Vec`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("2jiho").await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// if let Some(webtoons) = creator.webtoons().await? {
    ///     for webtoon in webtoons  {
    ///         println!("`{}`", webtoon.title());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
            Ok(response) if response.status() == 200 => response,
            Ok(_) => {
                let page = page(profile, &self.client).await?;
                let profile = page
                    .as_ref()
                    .map(|page| page.id.clone())
                    .context("failed to find creator profile property on creator page html")?;
                *self.page.write() = page;

                self.client.get_webtoons_from_creator_page(&profile).await?
            }
            Err(err) => return Err(CreatorError::ClientError(err)),
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

mod api {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Root {
        pub result: Result1,
        #[allow(unused)]
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
