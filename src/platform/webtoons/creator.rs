//! A creator on `webtoons.com`.

use super::{Client, Webtoon, error::CreatorError};
use crate::{
    platform::webtoons::error::{ClientError, CreatorWebtoonsError},
    stdx::{
        cache::{Cache, Store},
        macros::maybe,
    },
};
use assumptions::{Assume, Assumption, assume, assumption};
use core::fmt::{self, Debug};
use futures::future;
use scraper::{Html, Selector};

/// A creator on `webtoons.com`, obtained via [`Client::creator()`] or [`Webtoon::creators()`].
///
/// Only the English site is supported. However, some Original webtoons are authored
/// by Korean creators or studios (e.g. DC Comics) that have no `webtoons.com` account,
/// which means their profile page does not exist. Methods that require a profile page
/// return `None` in that case.
///
/// # Examples
///
/// ```
/// # use webtoon::platform::webtoons::{error::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let creator = client.creator("s0s2").await?.expect("`s0s2` exists");
///
/// assert_eq!("s0s2", creator.username());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Creator {
    pub(super) client: Client,
    pub(super) username: String,
    pub(super) profile: Option<String>,
    pub(super) homepage: Cache<Option<Homepage>>,
}

impl Debug for Creator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            client: _,
            username,
            profile,
            homepage,
        } = self;

        let mut debug = f.debug_struct("Creator");

        debug.field("profile", profile).field("username", username);

        if let Store::Value(Some(Homepage {
            username: _,
            followers,
            has_patreon,
            id,
        })) = homepage.get()
        {
            debug
                .field("id", &id)
                .field("followers", &followers)
                .field("has_patreon", &has_patreon);
        }

        debug.finish()
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
    /// Returns the username of this [`Creator`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("hanzaart").await?.expect("`hanzaart` exists");
    ///
    /// assert_eq!("Hanza Art", creator.username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn username(&self) -> &str {
        let creator = self;
        &creator.username
    }

    /// Returns the profile path segment used in `webtoons.com/*/creator/{profile}`, if any.
    ///
    /// Returns `None` for Korean or studio creators that have no `webtoons.com` account.
    ///
    /// Always `Some` when [`Creator`] was obtained via [`Client::creator()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("MaccusNormann").await?.expect("MaccusNormann exists");
    ///
    /// assert_eq!(Some("MaccusNormann"), creator.profile());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        let creator = self;
        creator.profile.as_deref()
    }

    /// Returns the internal id of this [`Creator`], if any.
    ///
    /// Returns `None` for Korean or studio creators that have no `webtoons.com` account,
    /// or if their profile page is disabled. Sometimes matches [`profile`](Creator::profile)
    /// with the leading `_` stripped.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("MaccusNormann").await?.expect("`MaccusNormann` exists");
    ///
    /// assert_eq!(Some("w7ml9"), creator.id().await?.as_deref());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn id(&self) -> Result<Option<String>, CreatorError> {
        let creator = self;
        let client = &self.client;

        match creator.homepage.get() {
            Store::Value(homepage) => Ok(homepage.map(|homepage| homepage.id)),
            Store::Empty if let Some(profile) = creator.profile.as_deref() => {
                let homepage = homepage(profile, client).await?;
                let id = homepage.as_ref().map(|homepage| homepage.id.clone());
                creator.homepage.insert(homepage);
                Ok(id)
            }
            Store::Empty => Ok(None),
        }
    }

    /// Returns the follower count for this [`Creator`], if any.
    ///
    /// Returns `None` for Korean or studio creators that have no `webtoons.com` account,
    /// or if their profile page is disabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("g8dak").await?.expect("`g8dak` exists");
    ///
    /// println!("{} has {:?} followers!", creator.username(), creator.followers().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn followers(&self) -> Result<Option<u32>, CreatorError> {
        let creator = self;
        let client = &self.client;

        match creator.homepage.get() {
            Store::Value(homepage) => Ok(homepage.map(|homepage| homepage.followers)),
            Store::Empty if let Some(profile) = creator.profile.as_deref() => {
                let homepage = homepage(profile, client).await?;
                let followers = homepage.as_ref().map(|page| page.followers);
                creator.homepage.insert(homepage);
                Ok(followers)
            }
            Store::Empty => Ok(None),
        }
    }

    /// Returns the [`Webtoon`]s this [`Creator`] is or was involved with, if any.
    ///
    /// Returns `None` for Korean or studio creators that have no `webtoons.com` account,
    /// or if their profile page is disabled. Only publicly visible webtoons are returned;
    /// may be an empty `Vec` if none are currently public.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("jayessart").await?.expect("`jayessart` exists");
    ///
    /// if let Some(webtoons) = creator.webtoons().await? {
    ///     for webtoon in webtoons  {
    ///         println!("{} is/was involved in making {}", creator.username(), webtoon.title().await?);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn webtoons(&self) -> Result<Option<Vec<Webtoon>>, CreatorWebtoonsError> {
        let creator = self;
        let client = &self.client;

        let Some(id) = creator.id().await? else {
            return Ok(None);
        };

        let response = creator.client.fetch_creator_webtoons(&id).await?;

        let titles = response.result.titles.iter();

        let webtoons = future::try_join_all(titles.map(|webtoon| async {
                let webtoon = match Webtoon::new_with_client(webtoon.id, webtoon.r#type, client).await {
                    Ok(Some(webtoon)) => webtoon,
                    Ok(None) => assumption!("`webtoons.com` creator webtoons API should only return ids for existing, public webtoons"),
                    Err(err) => return Err(err),
                };
                Ok::<Webtoon, ClientError>(webtoon)
            })).await?;

        maybe!(
            webtoons.is_empty(),
            "if Creator has no public Webtoons, then the returned list will be empty"
        );

        Ok(Some(webtoons))
    }

    /// Returns `true` if this [`Creator`] has a Patreon linked to their account, if any.
    ///
    /// Returns `None` for Korean or studio creators that have no `webtoons.com` account,
    /// or if their profile page is disabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let creator = client.creator("u8ehb").await?.expect("`u8ehb` exists");
    ///
    /// assert_eq!(creator.has_patreon().await?, Some(true));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn has_patreon(&self) -> Result<Option<bool>, CreatorError> {
        let creator = self;
        let client = &self.client;

        match creator.homepage.get() {
            Store::Value(homepage) => Ok(homepage.map(|homepage| homepage.has_patreon)),
            Store::Empty if let Some(profile) = creator.profile.as_deref() => {
                let homepage = homepage(profile, client).await?;
                let has_patreon = homepage.as_ref().map(|homepage| homepage.has_patreon);
                creator.homepage.insert(homepage);
                Ok(has_patreon)
            }
            Store::Empty => Ok(None),
        }
    }
}

pub(super) async fn homepage(
    profile: &str,
    client: &Client,
) -> Result<Option<Homepage>, CreatorError> {
    let Some(html) = client.fetch_creator_page(profile).await? else {
        return Ok(None);
    };

    if is_invalid(&html) {
        return Err(CreatorError::InvalidCreatorProfile);
    }

    if is_disabled_for_community_violation(&html) {
        return Ok(None);
    }

    Ok(Some(Homepage {
        username: username(&html)?,
        followers: followers(&html)?,
        has_patreon: has_patreon(&html),
        id: id(&html)?,
    }))
}

#[inline]
fn username(html: &Html) -> Result<String, Assumption> {
    let selector = Selector::parse(r#"head>meta[name="author"]"#)
        .expect(r#"`head>meta[name="author"]` should be a valid selector"#);

    if let Some(element) = html.select(&selector).next()
        && let Some(name) = element.value().attr("content")
    {
        return Ok(html_escape::decode_html_entities(name).to_string());
    }

    assumption!(
        r#"`head>meta[name="author"]` should be present on `webtoons.com` creator homepage"#
    );
}

#[inline]
fn followers(html: &Html) -> Result<u32, Assumption> {
    let selector = Selector::parse("span") //
        .expect("`span` should be a valid selector");

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
            .assumption(
                "follower count element on `webtoons.com` creator homepage should not be empty",
            )?
            .replace(',', "");

        assume!(
            !count.is_empty(),
            "text from creator followers metric on `webtoons.com` Creator homepage should never be empty"
        );

        assume!(
            count.chars().all(|d| d.is_ascii_digit()),
            "text from creator followers metric on `webtoons.com` Creator homepage should only be digits"
        );

        return count
            .parse::<u32>()
            .with_assumption( || format!("follower count in `CreatorBriefMetric_count` element should be plain digits or digits with commas, got: `{count}`"));
    }

    assumption!(
        "second `CreatorBriefMetric_count` span should be present on `webtoons.com` creator homepage"
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
#[inline]
fn id(html: &Html) -> Result<String, Assumption> {
    let selector = Selector::parse("script") //
        .expect("`script` should be a valid selector");

    for element in html.select(&selector) {
        if let Some(inner) = element.text().next()
            && let Some(idx) = inner.find("creatorId")
        {
            let id  = inner
                .get(idx..)
                .assumption(
                    "index from `find(\"creatorId\")` should always be a valid byte boundary in `webtoons.com` creator homepage script",
                )?
                // `creatorId\":\"n5z4d\"` -> `\":\"n5z4d\"`
                .trim_start_matches("creatorId")
                .chars()
                // Skips `\":\"` leaving `n5z4d\"`
                .skip_while(|ch| !ch.is_alphanumeric())
                // Takes `n5z4d`, stopping on `\` of `\"`, which we don't need.
                .take_while(|ch| ch.is_ascii_alphanumeric())
                .collect::<String>();

            assume!(
                !id.is_empty(),
                "`creatorId` on `webtoons.com` creator homepage should never be empty"
            );

            return Ok(id);
        }
    }

    assumption!(
        "should always find `creatorId` in script tag in `webtoons.com` Creator homepage html"
    )
}

#[inline]
#[must_use]
fn has_patreon(html: &Html) -> bool {
    let selector = Selector::parse("img") //
        .expect("`img` should be a valid selector");

    html.select(&selector)
        .filter_map(|element| element.value().attr("alt"))
        .any(|alt| alt == "PATREON")
}

// When a URL is invalid, the `webtoons.com` returns a 404. This can happen
// when the profile doesn't exist. It can also return a 200, but the page has
// an error message: `Invalid creator profile.`
//
// It's not exactly clear what the distinction is between these states is, but
// presumably the 404 means the profile doesn't exist, and the 200 + error message
// means that it does exist, but for some reason is reporting an error.
//
// https://www.webtoons.com/p/community/en/u/y87lz
#[inline]
#[must_use]
fn is_invalid(html: &Html) -> bool {
    let selector = Selector::parse("p") //
        .expect("`p` should be a valid selector");

    // Element(<p class="ErrorPage_text__FQYij">) => { Text("Invalid creator profile.") }
    html.select(&selector)
        .find(|element| {
            element
                .attr("class")
                .is_some_and(|class| class.starts_with("ErrorPage_text"))
        })
        .is_some_and(|element| {
            element
                .text()
                .next()
                .is_some_and(|text| text == "Invalid creator profile.")
        })
}

// When a creator page is disabled due to community policy violations, `webtoons.com`
// still returns a 200 status, so we must search for the text that is presented
// when its disabled.
//
// https://www.webtoons.com/p/community/en/u/_o2pgx6
#[inline]
#[must_use]
fn is_disabled_for_community_violation(html: &Html) -> bool {
    let selector = Selector::parse("p") //
        .expect("`p` should be a valid selector");

    html.select(&selector).any(|element| {
        element.text().next().is_some_and(|text| {
            text.starts_with(
                "This account has been disabled because it didn’t follow our community policy.",
            )
        })
    })
}
