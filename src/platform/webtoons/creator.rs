//! Module containing things related to a creator on `webtoons.com`.

use super::{Client, Language, Webtoon, error::CreatorError};
use crate::stdx::{
    cache::{Cache, Store},
    error::{Assume, AssumeFor, Assumption, assumption},
};
use core::fmt::{self, Debug};
use futures::future;
use scraper::{Html, Selector};

/// Represents a Creator of a `Webtoon`.
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
/// # use webtoon::platform::webtoons::{error::Error, Language, Client};
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
    pub(super) homepage: Cache<Option<Homepage>>,
}

impl Debug for Creator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            client: _,
            language,
            username,
            profile,
            homepage,
        } = self;

        f.debug_struct("Creator")
            .field("language", language)
            .field("username", username)
            .field("profile", profile)
            .field("homepage", homepage)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(super) struct Homepage {
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
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
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
    #[must_use]
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
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
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
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns a `Creators` id.
    ///
    /// Sometimes this is just the [`profile()`](Creator::profile()) with the `_` prefix stripped.
    ///
    /// If creator has no `webtoons.com` profile, then this will always return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("MaccusNormann", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some("w7ml9"), creator.id().await?.as_deref());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn id(&self) -> Result<Option<String>, CreatorError> {
        if let Store::Value(homepage) = self.homepage.get() {
            Ok(homepage.map(|homepage| homepage.id))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let homepage = homepage(self.language, profile, &self.client).await?;
            let id = homepage.as_ref().map(|homepage| homepage.id.clone());

            self.homepage.insert(homepage);

            Ok(id)
        }
    }

    /// Returns the number of followers for the `Creator`.
    ///
    /// Will return `None` if profile homepage is not a supported language:
    /// - French
    /// - German
    /// - Korean
    /// - Chinese.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("g8dak", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// println!("{} has {:?} followers!", creator.username(), creator.followers().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn followers(&self) -> Result<Option<u32>, CreatorError> {
        if let Store::Value(homepage) = self.homepage.get() {
            Ok(homepage.map(|homepage| homepage.followers))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let homepage = homepage(self.language, profile, &self.client).await?;
            let followers = homepage.as_ref().map(|page| page.followers);

            self.homepage.insert(homepage);

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
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("jayessart", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// if let Some(webtoons) = creator.webtoons().await? {
    ///     for webtoon in webtoons  {
    ///         println!("{} is/was involved in making {}", creator.username(), webtoon.title().await?);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn webtoons(&self) -> Result<Option<Vec<Webtoon>>, CreatorError> {
        let Some(profile) = self
            .profile
            .as_deref()
            // Profiles can be prefixed with `_` but the URL needs it trimmed to work.
            .map(|profile| profile.trim_start_matches('_'))
        else {
            return Ok(None);
        };

        let response = match self.client.creator_webtoons(profile, self.language).await {
            Ok(response) => response,
            // TODO: match on specific error variant that only represents that the profile used is wrong.
            // Err(CreatorWebtoonsError::WrongProfile) =>
            Err(_) => {
                let homepage = homepage(self.language, profile, &self.client).await?;

                let profile = homepage
                    .as_ref()
                    .map(|homepage| homepage.id.clone())
                    .assumption(
                        "failed to find creator profile property on creator homepage html",
                    )?;

                self.homepage.insert(homepage);

                self.client
                    .creator_webtoons(&profile, self.language)
                    .await?
            } // TODO: Err(err) => return Err(err.into()),
        };

        let webtoons =
            future::try_join_all(response.result.titles.iter().map(|webtoon| async {
                let webtoon = match Webtoon::new_with_client(webtoon.id, webtoon.r#type, &self.client).await {
                    Ok(Some(webtoon)) => webtoon,
                    Ok(None) => assumption!("`webtoons.com` creator homepage's webtoons API should return valid id's for existing and public webtoons"),
                    Err(err) => return Err(err.into()),
                };

                Ok::<Webtoon, CreatorError>(webtoon)
            })).await?;

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
    /// # use webtoon::platform::webtoons::{error::Error, Language, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(creator) = client.creator("u8ehb", Language::En).await? else {
    ///     unreachable!("profile is known to exist");
    /// };
    ///
    /// assert_eq!(Some(true), creator.has_patreon().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_patreon(&self) -> Result<Option<bool>, CreatorError> {
        if let Store::Value(homepage) = self.homepage.get() {
            Ok(homepage.map(|homepage| homepage.has_patreon))
        } else {
            let Some(profile) = self.profile.as_deref() else {
                return Ok(None);
            };

            let homepage = homepage(self.language, profile, &self.client).await?;
            let has_patreon = homepage.as_ref().map(|homepage| homepage.has_patreon);

            self.homepage.insert(homepage);

            Ok(has_patreon)
        }
    }
}

pub(super) async fn homepage(
    language: Language,
    profile: &str,
    client: &Client,
) -> Result<Option<Homepage>, CreatorError> {
    let Some(html) = client.creator_page(language, profile).await? else {
        return Ok(None);
    };

    Ok(Some(Homepage {
        username: username(&html)?,
        followers: followers(&html)?,
        has_patreon: has_patreon(&html)?,
        id: id(&html)?,
    }))
}

fn username(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse("h3").assumption("`h3` should be a valid selector")?;

    for element in html.select(&selector) {
        if let Some(class) = element.value().attr("class")
            && class.starts_with("HomeProfile_nickname")
        {
            return Ok(element
                .text()
                .next()
                .assumption("username element on `webtoons.com` creator homepage was empty")?
                .to_string());
        }
    }

    assumption!(
        "did not find any class that starts with `HomeProfile_nickname` on `webtoons.com` creator homepage html"
    );
}

fn followers(html: &Html) -> Result<u32, CreatorError> {
    let selector = Selector::parse("span").assumption("`span` should be a valid selector")?;

    if let Some(element) = html
        .select(&selector)
        // The same class name is used for series count as well.
        .filter(|element| {
            element
                .value()
                .attr("class")
                .is_some_and(|class| class.starts_with("CreatorBriefMetric_count"))
        })
        // To get the followers we need the second instance.
        .nth(1)
    {
        let count = element
            .text()
            .next()
            .assumption("follower count element on `weboons.com` creator homepage was empty")?
            .replace(',', "");

        return Ok(count
                    .parse::<u32>()
                    .assumption_for( |err| format!("follower count in `CreatorBriefMetric_count` element should always be either plain digits, or digits and commas, but got: {count}: {err}"))?);
    }

    assumption!(
        "did not find any class that starts with `CreatorBriefMetric_count` on `webtoons.com` creator homepage html"
    );
}

// In general, the profile id found at the end of the creator page URL is what
// is wanted. However, there are instances where there is a hidden backing profile
// id that is what is needed instead.
//
// Luckily, this hidden id can be found on the profile page itself, in a script
// tag in the HTML. This allows us to always take the public profile id and get
// access to the hidden id, if needed.
//
// Going the other way, however, is generally not possible. Luckily this shouldn't
// be needed.
fn id(html: &Html) -> Result<String, CreatorError> {
    let selector = Selector::parse("script").assumption("`script` should be a valid selector")?;

    for element in html.select(&selector) {
        if let Some(inner) = element.text().next()
            && let Some(idx) = inner.find("creatorId")
        {
            let id  = inner
                .get(idx..)
                .assumption(
                    "`find` should point to start of `webtoons.com` creator homepage `creatorId`, so should never be out of bounds",
                )?
                // `creatorId\":\"n5z4d\"` -> `\":\"n5z4d\"`
                .trim_start_matches("creatorId")
                .chars()
                // Skips `\":\"` leaving `n5z4d\"`
                .skip_while(|ch| !ch.is_alphanumeric())
                // Takes `n5z4d`, stopping on `\` of `\"`, which we don't need.
                .take_while(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>();

            assumption!(
                !id.is_empty(),
                "`creatorId` on `webtoons.com` creator homepage should never be empty"
            );

            return Ok(id);
        }
    }

    assumption!(
        "failed to find `creatorId` in script tag on english `webtoons.com` Creator homepage html"
    )
}

fn has_patreon(html: &Html) -> Result<bool, Assumption> {
    let selector = Selector::parse("img").assumption("`img` should be a valid selector")?;

    let has_patreon = html
        .select(&selector)
        .filter_map(|element| element.value().attr("alt"))
        .any(|alt| alt == "PATREON");

    Ok(has_patreon)
}
