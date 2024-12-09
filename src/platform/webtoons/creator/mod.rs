//! Module containing things related to a creator on webtoons.com.

use anyhow::{anyhow, Context};
use core::fmt::{self, Debug};
use scraper::{Html, Selector};

use super::{errors::CreatorError, Client, Language, Type, Webtoon};

// TODO: Implement page caching

/// Represents a creator of a webtoon.
///
/// More generally this represents an account on webtoons.com.
#[derive(Clone)]
pub struct Creator {
    pub(super) client: Client,
    pub(super) language: Language,
    pub(super) username: String,
    // Originals authors might not have a profile: Korean, Chinese, German, and French
    pub(super) profile: Option<String>,
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

pub(super) struct Page {
    pub username: String,
    pub followers: u32,
}

impl Creator {
    /// Returns a `Creators` username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a `Creators` profile segment in `https://www.webtoons.com/*/creator/{profile}`
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns the number of followers for the `Creator`.
    ///
    /// Will return `None` if profile page is not supported for language version.
    /// - French, German, Korean, and Chinese.
    pub async fn followers(&self) -> Result<Option<u32>, CreatorError> {
        let Some(profile) = self.profile.as_deref() else {
            return Ok(None);
        };

        let followers = page(self.language, profile, &self.client)
            .await?
            .map(|page| page.followers);

        Ok(followers)
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
        let Some(profile) = self.profile.as_deref() else {
            return Ok(None);
        };

        let url = format!("https://www.webtoons.com/p/community/api/v1/creator/{profile}/titles");

        let response = self
            .client
            .http
            .get(url)
            .send()
            .await?
            .json::<api::Response>()
            .await?;

        let mut webtoons = Vec::with_capacity(response.result.total_count);

        for webtoon in response.result.titles {
            let id = webtoon
                .id
                .parse::<u32>()
                .context("failed to parse webtoon id to number")?;

            let r#type = webtoon.r#type.parse::<Type>()?;

            webtoons.push(Webtoon::new_with_client(id, r#type, &self.client).await?);
        }

        Ok(Some(webtoons))
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
