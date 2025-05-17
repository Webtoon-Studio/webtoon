//! Module containing things related to a creator on `webtoons.com`.

use anyhow::{Context, anyhow};
use core::fmt::{self, Debug};
use parking_lot::RwLock;
use scraper::{Html, Selector};
use std::sync::Arc;

use super::{Client, Language, Type, Webtoon, errors::CreatorError};

/// Represents a creator of a `Webtoon`.
///
/// More generally this represents an account on `webtoons.com`.
///
/// This type is not constructed directly, instead it is gotten through a [`Client`] via [`Client::creator()`].
///
/// # Accounts and Languages
///
/// Not all languages support accounts, and the functionality of `Creator` will be more limited on those languages. This
/// is also true for Korean stories that have been brought over and translated. Rarely will the Korean creator have an
/// account on `webtoons.com`.
///
/// Some functionality works with such accountless creators, but it depends on the function. Read the method docs for more
/// info.
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(creator) = client.creator("s0s2", Language::En).await? else {
///     unreachable!("profile is known to exist");
/// };
///
/// assert_eq!("s0s2", creator.username());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Creator {
    pub(super) client: Client,
    pub(super) language: Language,
    pub(super) username: String,
    // Originals authors might not have a profile: Korean, Chinese, German, and French
    pub(super) profile: Option<String>,
    pub(super) page: Arc<RwLock<Option<Page>>>,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for Creator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creator")
            // omitting `client`
            .field("language", &self.language)
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
    pub has_patreon: bool,
}

impl Creator {
    /// Returns a `Creators` username.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("hanzaart", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!("Hanza Art", creator.username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a `Creators` profile segment in `https://www.webtoons.com/*/creator/{profile}`
    ///
    /// Not all creators for a story have a `webtoons.com` profile (Korean stories for example).
    ///
    /// - If constructed via [`Client::creator()`], then this will always be `Some`
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("MaccusNormann", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some("MaccusNormann"), creator.profile());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns a `Creators` id.
    ///
    /// Sometimes this is just the [`profile()`](Creator::profile()) with the `_` prefix stripped.
    ///
    /// If creator has no webtoon profile, then this will always return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("MaccusNormann", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some("w7ml9"), creator.profile().as_deref());
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

            let page = page(self.language, profile, &self.client).await?;
            let followers = page.as_ref().map(|page| page.id.clone());
            *self.page.write() = page;
            Ok(followers)
        }
    }

    /// Returns the number of followers for the `Creator`.
    ///
    /// Will return `None` if profile page is not a supported language:
    /// - French
    /// - German
    /// - Korean
    /// - Chinese.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("g8dak", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// println!("{} has {} followers!", creator.username(), creator.followers());
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

            let page = page(self.language, profile, &self.client).await?;
            let followers = page.as_ref().map(|page| page.followers);
            *self.page.write() = page;
            Ok(followers)
        }
    }

    /// Returns a list of [`Webtoon`] that the creator is/was involved with.
    ///
    /// # Returns
    ///
    /// Will return `Some` if there is a Webtoon profile, otherwise it will return `None`.
    ///
    /// This is for creators where there are no profile, either due to being a Korean based creator,
    /// or that the language version of `webtoons.com` does not support profile pages.
    ///
    /// The webtoons returned are only those that are publicly viewable. If there are no viewable webtoons, it will return an empty `Vec`.
    ///
    /// **Unsupported Languages**:
    /// - Korean
    /// - Chinese
    /// - French
    /// - German.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("jayessart", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// for webtoon in creator.webtoons().await? {
    ///     println!("{} is/was involved in making {}", creator.username(), webtoon.title().await?);
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

        let language = self.language.as_str_caps();

        let url = format!(
            "https://www.webtoons.com/p/community/api/v1/creator/{profile}/titles?language={language}"
        );

        let response = if let Ok(response) = self
            .client
            .http
            .get(url)
            .send()
            .await?
            .json::<api::Response>()
            .await
        {
            response
        } else {
            let page = page(self.language, profile, &self.client).await?;
            let profile = page
                .as_ref()
                .map(|page| page.id.clone())
                .context("failed to find creator profile property on creator page html")?;
            *self.page.write() = page;

            let url = format!(
                "https://www.webtoons.com/p/community/api/v1/creator/{profile}/titles?language={language}"
            );

            self.client
                .http
                .get(url)
                .send()
                .await?
                .json::<api::Response>()
                .await?
        };

        let mut webtoons = Vec::with_capacity(response.result.total_count);

        for webtoon in response.result.titles {
            let id = webtoon
                .id
                .parse::<u32>()
                .context("failed to parse webtoon id to number")?;

            let r#type: Type = webtoon.r#type.parse()?;

            webtoons.push(Webtoon::new_with_client(id, r#type, &self.client).await?);
        }

        Ok(Some(webtoons))
    }

    /// Returns if creator has a `Patreon` linked to their account.
    ///
    /// Will return `None` if the language version of the site doesn't support profile pages.
    ///
    /// **Unsupported Languages**:
    /// - Korean
    /// - Chinese
    /// - French
    /// - German.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("u8ehb", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert!(creator.has_patreon().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_patreon(&self) -> Result<Option<bool>, CreatorError> {
        if let Some(page) = &*self.page.read() {
            Ok(Some(page.has_patreon))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let page = page(self.language, profile, &self.client).await?;
            let has_patreon = page.as_ref().map(|page| page.has_patreon);
            *self.page.write() = page;
            Ok(has_patreon)
        }
    }
}

pub(super) async fn page(
    language: Language,
    profile: &str,
    client: &Client,
) -> Result<Option<Page>, CreatorError> {
    let response = client.get_creator_page(language, profile).await?;

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
        has_patreon: has_patreon(&html),
        id: id(&html)?,
    }))
}

fn username(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse("h3").expect("`h3` should be a valid selector");

    for element in html.select(&selector) {
        // TODO: When Rust 2024 comes out with let chains, then switch to that, rather than nested like this.
        if let Some(class) = element.value().attr("class") {
            if class.starts_with("HomeProfile_nickname") {
                return Ok(element
                    .text()
                    .next()
                    .context("username element was empty")?
                    .to_string());
            }
        }
    }

    Err(CreatorError::Unexpected(anyhow!(
        "failed to find creator username on creator page"
    )))
}

fn followers(html: &Html) -> Result<u32, CreatorError> {
    let selector = Selector::parse("span").expect("`span` should be a valid selector");

    // The same class name is used for series count as well. To get the followers, we need the second instance,
    let mut encountered_class = false;

    for element in html.select(&selector) {
        // TODO: When Rust 2024 comes out with let chains, then switch to that, rather than nested like this.
        if let Some(class) = element.value().attr("class") {
            if class.starts_with("CreatorBriefMetric_count") {
                if encountered_class {
                    return Ok(element
                        .text()
                        .next()
                        .context("follower count element was empty")?
                        .replace(',', "")
                        .parse()
                        .context("follower count was not a number")?);
                }

                encountered_class = true;
            }
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

fn has_patreon(html: &Html) -> bool {
    let selector = Selector::parse("img").expect("`img` should be a valid selector");

    let mut has_patreon = false;

    for element in html.select(&selector) {
        // TODO: When Rust 2024 comes out with let chains, then switch to that, rather than nested like this.
        if let Some(alt) = element.value().attr("alt") {
            if alt == "PATREON" {
                has_patreon = true;
                break;
            }
        }
    }

    has_patreon
}

#[allow(unused)]
mod api {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct Response {
        pub result: Result,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct Result {
        pub titles: Vec<Titles>,
        pub total_count: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct Titles {
        pub id: String,
        #[serde(rename = "subject")]
        pub title: String,
        pub authors: Vec<Authors>,
        pub genres: Vec<String>,
        #[serde(rename = "grade")]
        pub r#type: String,
        pub thumbnail_url: String,
        pub recent_episode_registered_at: i64,
        pub title_registered_at: i64,
    }

    #[derive(Deserialize)]
    pub(super) struct Authors {
        pub nickname: String,
    }
}
