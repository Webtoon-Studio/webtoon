//! Module containing things related to a creator on webtoons.com.

use anyhow::Context;
use core::fmt::{self, Debug};
use scraper::{Html, Selector};

use super::{errors::CreatorError, Client, Language, Webtoon};

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
    pub webtoons: Vec<Webtoon>,
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

        let webtoons = page(self.language, profile, &self.client)
            .await?
            .map(|page| page.webtoons);

        Ok(webtoons)
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

    let document = response.text().await?;

    let html = Html::parse_document(&document);

    Ok(Some(Page {
        username: username(&html)?,
        webtoons: webtoons(&html, client)?,
        followers: followers(&html)?,
    }))
}

fn username(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse("div.author_wrap>div.info>strong.nickname")
        .expect("`div.author_wrap>div.info>strong.nickname` should be a valid selector");

    let username = html
        .select(&selector)
        .next()
        .context("username was not found on page")?
        .text()
        .next()
        .context("username element was empty")?
        .to_string();

    Ok(username)
}

fn followers(html: &Html) -> Result<u32, CreatorError> {
    let selector = Selector::parse("div.author_wrap>div.info>div.etc>strong>span._followerCount")
        .expect(
        "`div.author_wrap>div.info>div.etc>strong>span._followerCount` should be a valid selector",
    );

    let followers: u32 = html
        .select(&selector)
        .next()
        .context("follower count was not found on page")?
        .text()
        .next()
        .context("follower count element was empty")?
        .replace(',', "")
        .parse()
        .context("follower count was not a number")?;

    Ok(followers)
}

fn webtoons(html: &Html, client: &Client) -> Result<Vec<Webtoon>, CreatorError> {
    let mut webtoons = Vec::new();

    let selector = Selector::parse("ul.work_list>li.item>a._clickTitleGaLogging")
        .expect("`ul.work_list>li.item>a._clickTitleGaLogging` should be a valid selector");

    for element in html.select(&selector) {
        let url = element.attr("href") //
                .context("`href` is missing, `ul.work_list>li.item>a._clickTitleGaLogging` should always have one")?;

        webtoons.push(Webtoon::from_url_with_client(url, client)?);
    }

    Ok(webtoons)
}
