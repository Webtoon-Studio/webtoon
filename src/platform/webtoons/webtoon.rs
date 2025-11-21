//! Represents an abstraction for a Webtoon on `webtoons.com`.

mod homepage;

pub mod episode;
pub mod post;

use core::fmt::{self, Debug};
use parking_lot::RwLock;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(feature = "rss")]
pub mod rss;
#[cfg(feature = "rss")]
use rss::Rss;

use crate::{
    platform::webtoons::error::InvalidWebtoonUrl,
    stdx::{
        error::{Invariant, invariant},
        http::IRetry,
    },
};

use self::{
    episode::{Episode, Episodes},
    homepage::Page,
    post::Posts,
};

use super::Type;
use super::error::{ClientError, EpisodeError, PostError, WebtoonError};
use super::meta::{Genre, Scope};
use super::originals::Schedule;
use super::{Client, Language, creator::Creator};

/// Represents a Webtoon from `webtoons.com`.
///
/// This type is not constructed directly, instead it is gotten through a [`Client`] via [`Client::webtoon()`] or [`Client::webtoon_from_url()`].
///
/// This abstracts over all the sections a Webtoon may be in, such as the `originals` or `canvas` sections. Relevant capabilities
/// are exposed, with methods taking missing features that may not exists across all sections into account.
///
/// Read the method documentation for more info.
#[derive(Clone)]
pub struct Webtoon {
    pub(super) client: Client,
    pub(super) id: u32,
    pub(super) language: Language,
    // some genre for an original or canvas for canvas webtoons: "fantasy" or "canvas"
    pub(super) scope: Scope,
    /// url slug of the webtoon name: Tower of God -> tower-of-god
    pub(super) slug: Arc<str>,
    /// Cache for data on the Wetboons landing page: title, etc.
    pub(super) page: Arc<RwLock<Option<Page>>>,
}

impl Debug for Webtoon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            client: _,
            id,
            language,
            scope,
            slug,
            page,
        } = self;

        f.debug_struct("Webtoon")
            .field("id", id)
            .field("language", language)
            .field("scope", scope)
            .field("slug", slug)
            .field("page", page)
            .finish()
    }
}

impl Webtoon {
    /// Returns the [`Language`] of this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client, Language};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1817, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(Language::En, webtoon.language());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn language(&self) -> Language {
        self.language
    }

    /// Returns the id of this `Webtoon`.
    ///
    /// This corresponds to the `title_no` query: `https://www.webtoons.com/en/fantasy/osora/list?title_no=6202`
    ///
    /// Most often this will be what was just passed in directly.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6202, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(6202, webtoon.id());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the type of this `Webtoon`: [`Original`](variant@Type::Original) or [`Canvas`](variant@Type::Canvas).
    ///
    /// For doing simple boolean checks, prefer [`is_canvas()`](Webtoon::is_canvas()) or [`is_original()`](Webtoon::is_original()).
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6880, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(Type::Original, webtoon.r#type());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn r#type(&self) -> Type {
        match self.scope {
            Scope::Original(_) => Type::Original,
            Scope::Canvas => Type::Canvas,
        }
    }

    /// Returns if `Webtoon` is an [`Original`](variant@Type::Original) type.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6880, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_original());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_original(&self) -> bool {
        self.r#type() == Type::Original
    }

    /// Returns if `Webtoon` is a [`Canvas`](variant@Type::Canvas) type.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(991073, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_canvas());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_canvas(&self) -> bool {
        self.r#type() == Type::Canvas
    }

    /// Returns the title of this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6880, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!("I Became a Level 999 Demon Queen", webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn title(&self) -> Result<String, WebtoonError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.title().to_string())
        } else {
            let page = homepage::scrape(self).await?;
            let title = page.title().to_owned();
            *self.page.write() = Some(page);
            Ok(title)
        }
    }

    /// Returns a list of [`Creator`] for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(794611, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let creators = webtoon.creators().await?;
    ///
    /// assert!(creators.len() == 1);
    /// assert_eq!("AlmightyConurbano", creators[0].username());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn creators(&self) -> Result<Vec<Creator>, WebtoonError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.creators().to_vec())
        } else {
            let page = homepage::scrape(self).await?;
            let creators = page.creators().to_vec();
            *self.page.write() = Some(page);
            Ok(creators)
        }
    }

    /// Returns a list of [`Genre`] for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, meta::Genre, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1039653, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let genres = webtoon.genres().await?;
    ///
    /// assert!(genres.len() == 2);
    /// assert_eq!(Genre::Action, genres[0]);
    /// assert_eq!(Genre::Fantasy, genres[1]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn genres(&self) -> Result<Vec<Genre>, WebtoonError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.genres().to_vec())
        } else {
            let page = homepage::scrape(self).await?;
            let genres = page.genres().to_vec();
            *self.page.write() = Some(page);
            Ok(genres)
        }
    }

    /// Returns the summary for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(95, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!("What do you desire? Money and wealth? Honor and pride? Authority and power? Revenge? Or something that transcends them all? Whatever you desire—it's here.", webtoon.summary().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn summary(&self) -> Result<String, WebtoonError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.summary().to_owned())
        } else {
            let page = homepage::scrape(self).await?;
            let summary = page.summary().to_owned();
            *self.page.write() = Some(page);
            Ok(summary)
        }
    }

    /// Retrieves the total number of views for this `Webtoon`.
    ///
    /// # Behavior
    /// The method determines the total views based on whether the current session belongs to the creator of the webtoon:
    ///
    /// - **Without Creator Session**: If the current session does not belong to the webtoon creator, or if no session is available, this method returns the view count from the webtoon's main page. This value may be rounded (e.g., `3,800,000`).
    /// - **With Creator Session**: If the session belongs to the creator of the webtoon, this method fetches more detailed episode-by-episode view counts and sums them to return a more accurate total view count (e.g., `3,804,237` instead of `3,800,000`).
    ///
    /// **ONLY ENGLISH DASHBOARD SUPPORTED**
    /// - Even if valid session is provided for the webtoon creator, only the public data on the Webtoon's page will be gotten.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1320, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(1_400_000_000, webtoon.views().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn views(&self) -> Result<u64, EpisodeError> {
        match self.client.get_user_info_for_webtoon(self).await {
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                let views = super::dashboard::episodes::scrape(self)
                    .await?
                    .into_iter()
                    .filter_map(|episode| episode.views.map(u64::from))
                    .sum::<u64>();

                return Ok(views);
            }
            // Fallback to public data
            Ok(_) | Err(ClientError::NoSessionProvided) => {}
            Err(err) => return Err(EpisodeError::ClientError(err)),
        }

        if let Some(page) = &*self.page.read() {
            Ok(page.views())
        } else {
            let page = homepage::scrape(self).await.map_err(|err| match err {
                WebtoonError::ClientError(client_error) => EpisodeError::ClientError(client_error),
                error => EpisodeError::Unexpected(error.into()),
            })?;
            let views = page.views();
            *self.page.write() = Some(page);
            Ok(views)
        }
    }

    /// Retrieves the total number of subscribers for this `Webtoon`.
    ///
    ///
    /// # Behavior
    /// The method determines the subscriber count based on whether the current session belongs to the creator of the webtoon:
    ///
    /// - **Without Creator Session**: If the current session does not belong to the webtoon creator, or if no session is available, this method returns the subscriber count from the webtoon's main page. This count may be less precise, as it can be a rounded number.
    /// - **With Creator Session**: If the session belongs to the creator of the webtoon, this method retrieves subscriber statistics directly from the creator's stats dashboard, providing a more accurate and up-to-date count of subscribers.
    ///
    /// **ONLY ENGLISH DASHBOARDS SUPPORTED**
    /// - If you use with a non-english webtoon, even with a valid session provided that is of the owner of the webtoon it will get the public facing page data.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1436, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(7_500_000, webtoon.subscribers().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribers(&self) -> Result<u32, WebtoonError> {
        match self.client.get_user_info_for_webtoon(self).await {
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                let subscribers = super::dashboard::statistics::scrape(self)
                    .await?
                    .subscribers;
                return Ok(subscribers);
            }
            // Fallback to public data
            Ok(_) | Err(ClientError::NoSessionProvided) => {}
            Err(err) => return Err(WebtoonError::ClientError(err)),
        }

        if let Some(page) = &*self.page.read() {
            Ok(page.subscribers())
        } else {
            let page = homepage::scrape(self).await?;
            let subscribers = page.subscribers();
            *self.page.write() = Some(page);
            Ok(subscribers)
        }
    }

    /// Returns the thumbnail `url` for this `Webtoon`.
    ///
    /// If `Webtoon` is an [`Original`](variant@Type::Original), this will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(19985, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(thumbnail) = webtoon.thumbnail().await? {
    ///     println!("thumbnail for {} can be found here {thumbnail}", webtoon.title().await?);
    ///     # return Ok(());
    /// }
    ///
    /// # unreachable!("should have entered the thumbail block");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn thumbnail(&self) -> Result<Option<String>, WebtoonError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.thumbnail().map(|thumbnail| thumbnail.to_string()))
        } else {
            let page = homepage::scrape(self).await?;
            let thumbnail = page.thumbnail().map(|thumbnail| thumbnail.to_string());
            *self.page.write() = Some(page);
            Ok(thumbnail)
        }
    }

    /// Retrieves the release schedule for this `Webtoon`.
    ///
    /// # Behavior
    ///
    /// - **Original Webtoons**: If the webtoon is an Original series, this method fetches the release schedule and returns it as a `Some(Schedule)`. The release schedule contains information about upcoming or regular episode drops.
    /// - **Canvas Webtoons**: If the webtoon is part of the Canvas section, there is no official release schedule, and this method will return `None` and cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, originals::Schedule, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// match webtoon.schedule().await? {
    ///     Some(Schedule::Weekday(weekday)) => println!("Webtoon releases on `{weekday:?}`"),
    ///     Some(Schedule::Daily) => println!("Webtoon releases daily"),
    ///     Some(Schedule::Completed) => println!("Webtoon is completed"),
    ///     Some(Schedule::Weekdays(weekdays)) => {
    ///        print!("Webtoon releases on ");
    ///        for day in weekdays {
    ///            print!("{day:?}");
    ///        }
    ///        println!();
    ///     }
    ///     None => println!("This webtoon does not have a release schedule (Canvas)."),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn schedule(&self) -> Result<Option<Schedule>, WebtoonError> {
        if self.r#type() == Type::Canvas {
            return Ok(None);
        }

        if let Some(page) = &*self.page.read() {
            Ok(page.schedule().cloned())
        } else {
            let page = homepage::scrape(self).await?;
            let release = page.schedule().cloned();
            *self.page.write() = Some(page);
            Ok(release)
        }
    }

    /// Returns if `Webtoon` is completed.
    ///
    /// Canvas stories always return `false` and cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(93, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_completed().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_completed(&self) -> Result<bool, WebtoonError> {
        if self.r#type() == Type::Canvas {
            return Ok(false);
        }

        if let Some(page) = &*self.page.read() {
            Ok(page.schedule() == Some(&Schedule::Completed))
        } else {
            let page = homepage::scrape(self).await?;
            let is_completed = page.schedule() == Some(&Schedule::Completed);
            *self.page.write() = Some(page);
            Ok(is_completed)
        }
    }

    /// Retrieves the banner image URL for this `Webtoon`.
    ///
    /// # Behavior
    ///
    /// - **Original Webtoons**: If the webtoon is an Original series, this method returns the URL of the banner image, which is typically displayed at the top of the webtoon's main page.
    /// - **Canvas Webtoons**: Canvas webtoons do not have banner images, so the method will return `None` if the webtoon is in the Canvas section and cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(3349, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("banner for {} can be found here {:?}", webtoon.title().await?, webtoon.banner().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn banner(&self) -> Result<Option<String>, WebtoonError> {
        if self.scope == Scope::Canvas {
            return Ok(None);
        }

        if let Some(page) = &*self.page.read() {
            Ok(page.banner().map(|banner| banner.to_owned()))
        } else {
            let page = homepage::scrape(self).await?;
            let release = page.banner().map(|release| release.to_owned());
            *self.page.write() = Some(page);
            Ok(release)
        }
    }

    /// Retrieves all episodes for the current `Webtoon`.
    ///
    /// The method's behavior depends on whether the user (of the session) is the creator of the webtoon or a regular
    /// viewer (i.e., a session is provided or not, and if the session user is a creator for the webtoon).
    ///
    /// **ONLY ENGLISH DASHBOARD SUPPORTED**
    /// - Even if valid session is provided for the webtoon creator, only the public data will be gotten.
    ///
    /// The episodes are returned in descending order.
    ///
    /// # Behavior
    ///
    /// - **For Creators**:
    ///     - If the session belongs to the creator of the webtoon, the method will scrape the episode dashboard. This includes:
    ///         - All episodes, including drafts.
    ///         - Episodes behind fast-pass or ad walls.
    ///         - Access to additional episode metadata, such as view counts (`Episode::views()` will return `Some(u32)` for episodes retrieved via the dashboard).
    ///         - Fully accurate publication times (`Episode::published()` will return the exact timestamp of the episode's release).
    ///
    /// - **For Regular Users**:
    ///     - If the session is not provided or the user is not the creator of the webtoon, the method will scrape the publicly available episodes:
    ///         - Only episodes that are publicly visible on the webtoon's main page will be retrieved.
    ///         - Episodes behind fast-pass or ad walls will not be included.
    ///         - View counts (`Episode::views()`) will return `None` for episodes retrieved from the main page as the information is unavailable.
    ///         - The publication time (`Episode::published()`) will return `Some(i64)` but the time will always be set to `2:00 AM UTC` on the episode's published date.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1018, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// assert_eq!(25, episodes.count());
    ///
    /// for episode in episodes {
    ///     println!("title: {}", episode.title().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn episodes(&self) -> Result<Episodes, EpisodeError> {
        let episodes = match self.client.get_user_info_for_webtoon(self).await {
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                super::dashboard::episodes::scrape(self).await?
            }
            // Fallback to public data
            Ok(_) | Err(ClientError::NoSessionProvided) => {
                homepage::episodes(self).await.map_err(|err| match err {
                    WebtoonError::ClientError(client_error) => {
                        EpisodeError::ClientError(client_error)
                    }
                    error => EpisodeError::Unexpected(error.into()),
                })?
            }
            Err(err) => return Err(EpisodeError::ClientError(err)),
        };

        Ok(Episodes {
            count: u16::try_from(episodes.len())
                .map_err(|err| EpisodeError::Unexpected(err.into()))?,
            episodes,
        })
    }

    /// Constructs an `Episode` if it exists.
    ///
    /// However, there are important caveats to be aware of when using this method instead of `episodes`.
    ///
    /// # Caveats
    ///
    /// - **No View or Publish Data**:
    ///     - `Episode::views()` will always return `None`. This method does not provide view counts; you must use `episodes()` to retrieve views.
    ///     - `Episode::published()` will always return `None`. To get publication dates, you must use `episodes()`. This is a limitation as currently there is no known way to get the published date with just the episode value alone.
    ///
    /// - **Episode Existence vs. Public Display**:
    ///     - This method includes episodes that are unpublished, behind ads or fast-pass, or even "deleted" (i.e., episodes that no longer appear on the main page but are still accessible through their episode number).
    ///     - It does not rely solely on public episodes, meaning it will count and retrieve episodes that a regular user would not normally see without having access to a creator's dashboard or matching creator-webtoon session.
    ///     - The numbering (`#NUMBER`) of episodes retrieved by this method may differ from public episode lists due to the inclusion of hidden or removed episodes. You can see the matching episode with the `episode_no=` query in the URL.
    ///
    /// # Use Cases
    ///
    /// - Accessing hidden episodes, such as those unpublished, behind fast-pass, ad-walled, or deleted, without requiring a matching creator session.
    /// - Useful for situations where the complete set of episodes is necessary, including drafts or episodes not currently visible to the public.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(95, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// // Known hidden episode.
    /// if let Some(episode) = webtoon.episode(121).await? {
    ///     assert_eq!("[Season 2] Ep. 41", episode.title().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn episode(&self, number: u16) -> Result<Option<Episode>, EpisodeError> {
        let episode = Episode::new(self, number);

        if !episode.exists().await? {
            return Ok(None);
        }

        Ok(Some(episode))
    }

    /// A specialization for quickly getting the first publicly published episode.
    ///
    /// - Differing from `episodes` in that it doesn't traverse other pages to get the episodes.
    /// - Differing from `episode` this gets episode data like the published date.
    ///
    /// This is most useful for trying to see when a Webtoon started publishing, for example,
    /// to get all Originals who are "new", which is any Webtoon with a first episode published
    /// in the last 30 days.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(4176, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episode = webtoon.first_episode().await?;
    ///
    /// assert_eq!("Ep. 1 - The Return (1)", episode.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn first_episode(&self) -> Result<Episode, WebtoonError> {
        homepage::first_episode(self).await
    }

    /// Retrieves the total number of likes for all episodes of the current `Webtoon`.
    ///
    /// # Behavior
    ///
    /// This includes episodes behind ads, fast-pass, or are even deleted. This can lead to a discrepancy between the publicly displayed episodes and the actual total likes, as it accounts for episodes that are normally hidden or restricted from public view.
    ///
    /// - This method sums the likes across all episodes, regardless of whether the episodes are:
    ///   - **Publicly available**: It includes public episodes that any user can see.
    ///   - **Hidden**: Episodes behind fast-pass or ad walls.
    ///   - **Deleted**: Episodes that no longer appear on the public main page but still exist in the system.
    ///   - **Unpublished**: Drafts or episodes not yet publicly released.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(7666, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("{} has {} total likes!", webtoon.title().await?, webtoon.likes().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn likes(&self) -> Result<u32, EpisodeError> {
        let mut likes = 0;
        for number in 1.. {
            if let Some(episode) = self.episode(number).await? {
                likes += episode.likes().await?;
            } else {
                break;
            }
        }

        Ok(likes)
    }

    /// Retrieves all posts (top level comments) for every episode of the current `Webtoon`.
    ///
    /// # Behavior
    ///
    /// This method can return more posts than what is publicly available on the episode page, as it includes certain deleted posts, as well as those visible to all users.
    ///
    /// - This retrieves all posts across every episode, including:
    ///   - **Publicly visible posts**: Comments that any user can see on the webtoon page.
    ///   - **Deleted posts with replies**: Posts that have been marked as deleted but still display the message "This comment has been deleted" because they have replies.
    ///   - **Excluded posts**: Deleted posts without any replies are not included in the results.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(886472, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// for post in webtoon.posts().await? {
    ///    println!("{} left a post on {}!", post.poster().username(), webtoon.title().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts(&self) -> Result<Posts, PostError> {
        let mut posts = Vec::new();

        for number in 1.. {
            if let Some(episode) = self.episode(number).await.map_err(|err| match err {
                EpisodeError::ClientError(client_error) => PostError::ClientError(client_error),
                error => PostError::Unexpected(error.into()),
            })? {
                posts.extend_from_slice(episode.posts().await?.as_slice());
            } else {
                break;
            }
        }

        Ok(posts.into())
    }

    /// Retrieves the RSS feed information for the current `Webtoon`.
    ///
    /// This includes data for recently published episodes, but excludes episodes that are behind fast-pass or ad walls.
    ///
    /// **CURRENTLY ONLY ENGLISH IS SUPPORTED**
    ///
    /// # Behavior
    ///
    /// - **Episode Data**: The feed includes only episodes that are publicly and freely available, without fast-pass or ad restrictions.
    /// - **Thumbnails**: Instead of including the links to the first two panels of the episode as Webtoons.com provides, this method returns a list of `Episode`s where `Episode::thumbnail` can be used, for example. This design choice aligns more closely with user expectations for an RSS feed, making the implementation more intuitive for feed readers.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(7666, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let rss = webtoon.rss().await?;
    ///
    /// assert_eq!("The White Tower’s Rogue Mage", rss.title());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "rss")]
    pub async fn rss(&self) -> Result<Rss, WebtoonError> {
        rss::feed(self).await
    }

    /// Checks if the current user session is subscribed to the `Webtoon`.
    ///
    /// # Session
    ///
    /// This method requires a valid user session to perform actions on the Webtoon.
    ///   - If the session is invalid, it will return an error of type `Err(WebtoonError::ClientError(ClientError::InvalidSession))`.
    ///   - If no session is provided, it will return an error of type `Err(WebtoonError(ClientError::NoSessionProvided))`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("my-session");
    ///
    /// let Some(webtoon) = client.webtoon(2159, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if webtoon.is_subscribed().await? {
    ///    println!("Already subscribed to {}!", webtoon.title().await?);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_subscribed(&self) -> Result<bool, WebtoonError> {
        Ok(self.client.get_user_info_for_webtoon(self).await?.favorite)
    }

    /// Subscribes the current user to the `Webtoon`, if not already subscribed.
    ///
    /// # Session
    ///
    /// This method requires a valid user session to perform actions on the Webtoon.
    ///   - If the session is invalid, it will return an error of type `Err(WebtoonError::ClientError(ClientError::InvalidSession))`.
    ///   - If no session is provided, it will return an error of type `Err(WebtoonError(ClientError::NoSessionProvided))`.
    ///
    /// # Behavior
    ///
    /// - **Creator Check**: Checks if the user is the creator of the Webtoon. If the user is the creator, the method does nothing and immediately returns `Ok(())`.
    /// - **Subscription Status Check**: If the user is already subscribed, returns `Ok(())` without taking any further action.
    /// - **Subscribing**: If successful, returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("my-session");
    ///
    /// let Some(webtoon) = client.webtoon(7829, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if webtoon.subscribe().await.is_ok() {
    ///    println!("Subscribed to {}!", webtoon.title().await?);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&self) -> Result<(), WebtoonError> {
        let user = self.client.get_user_info_for_webtoon(self).await?;

        // Can't sub to own Webtoon
        if user.is_webtoon_creator() {
            return Ok(());
        }

        // Already subscribed
        if user.favorite {
            return Ok(());
        }

        self.client.post_subscribe_to_webtoon(self).await?;

        Ok(())
    }

    /// Unsubscribes the current user from the `Webtoon`, if currently subscribed.
    ///
    /// # Session
    ///
    /// This method requires a valid user session to perform actions on the Webtoon.
    ///   - If the session is invalid, it will return an error of type `Err(WebtoonError::ClientError(ClientError::InvalidSession))`.
    ///   - If no session is provided, it will return an error of type `Err(WebtoonError(ClientError::NoSessionProvided))`.
    ///
    /// # Behavior
    ///
    /// - **Creator Check**: Checks if the user is the creator of the Webtoon. If the user is the creator, the method does nothing and returns `Ok(())`.
    /// - **Subscription Status Check**: If the user is not subscribed, the method returns `Ok(())` without taking any further action.
    /// - **Unsubscribing**: If successful, returns `Ok(())`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("my-session");
    ///
    /// let Some(webtoon) = client.webtoon(7829, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if webtoon.unsubscribe().await.is_ok() {
    ///    println!("Unubscribed to {} 😞", webtoon.title().await?);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&self) -> Result<(), WebtoonError> {
        let title_user_info = self.client.get_user_info_for_webtoon(self).await?;

        // Can't sub to own webtoon so also can't unsub
        if title_user_info.is_webtoon_creator() {
            return Ok(());
        }

        // Already not subscribed
        if !title_user_info.favorite {
            return Ok(());
        }

        self.client.post_unsubscribe_to_webtoon(self).await?;

        Ok(())
    }
}

// Internal use
impl Webtoon {
    pub(super) async fn new_with_client(
        id: u32,
        r#type: Type,
        client: &Client,
    ) -> Result<Option<Self>, WebtoonError> {
        let url = format!(
            "https://www.webtoons.com/*/{}/*/list?title_no={id}",
            match r#type {
                Type::Original => "*",
                Type::Canvas => "canvas",
            }
        );

        let response = client.http.get(&url).retry().send().await?;

        // Webtoon doesn't exist or is not public.
        if response.status() == 404 {
            return Ok(None);
        }

        let url = response.url();

        let mut segments = url.path_segments().invariant(
            format!("the returned url from `webtoons.com` should have path segments (`/`); this url did not: `{url}`"),
        )?;

        let lang = segments
            .next()
            .invariant("`webtoons.com` returned url has path segments, but for some reason failed to extract the first segment, which should be a language: e.g `en`")?;

        let language = match Language::from_str(lang) {
            Ok(langauge) => langauge,
            Err(err) => invariant!(
                "first segement of the `webtoons.com` returned url provided an unexpected language: {err}"
            ),
        };

        let scope = segments
            .next()
            .invariant("`webtoons.com` returned url didn't have a second segment, representing the scope of the Webtoon")?;

        let scope = match Scope::from_str(scope) {
            Ok(scope) => scope,
            Err(err) => {
                invariant!(
                    "`webtoons.com` returned url's third segment provided an unexpected scope: {err}"
                )
            }
        };

        let slug = segments
            .next()
            .invariant("`webtoons.com` returned url didn't have a third segment, representing the slug name of the Webtoon")?
            .to_string();

        let webtoon = Webtoon {
            client: client.clone(),
            id,
            language,
            scope,
            slug: Arc::from(slug),
            page: Arc::new(RwLock::new(None)),
        };

        Ok(Some(webtoon))
    }

    pub(super) fn from_url_with_client(
        url: &str,
        client: &Client,
    ) -> Result<Self, InvalidWebtoonUrl> {
        let Ok(url) = url::Url::parse(url) else {
            return Err(InvalidWebtoonUrl::new(
                "failed to parse provided url: not a valid url",
            ));
        };

        let mut segments = url
            .path_segments() //
            .ok_or(InvalidWebtoonUrl::new("a `webtoons.com` Webtoon homepage url should have segments (`/`); this url did not"))?;

        let lang = segments
            .next()
            .ok_or(InvalidWebtoonUrl::new("url has path segments, but for some reason failed to extract the first segment, which for a valid `webtoons.com` Webtoon homepage url, should be a language: e.g `en`"))?;

        let language = match Language::from_str(lang) {
            Ok(language) => language,
            Err(err) => {
                return Err(InvalidWebtoonUrl::new(format!(
                    "found an unexpected language in provided `webtoons.com` url: {err}"
                )));
            }
        };

        let segment = segments
            .next() //
            .ok_or(InvalidWebtoonUrl::new("provided url didn't have a second segment, representing the scope of a Webtoon in a valid `webtoons.com` homepage url, eg. `canvas`, `fantasy`, etc."))?;

        let scope = match Scope::from_str(segment) {
            Ok(scope) => scope,
            Err(err) => {
                return Err(InvalidWebtoonUrl::new(format!(
                    "found an unexpected scope in provided `webtoons.com` url: {err}"
                )));
            }
        };

        let slug = segments
            .next()
            .ok_or( InvalidWebtoonUrl::new( "provided url didn't have a third segment, representing the slug name of a Webtoon in a valid `webtoons.com` homepage url, eg. `tower-of-god`"))?;

        let query = url
            .query()
            .ok_or(InvalidWebtoonUrl::new("a valid `webtoons.com` Webtoon homepage url should have a `title_no` query, but provided url didn't have any queries at all"))?;

        let id = match query.split_once('=') {
            Some(("title_no", "")) => {
                return Err(InvalidWebtoonUrl::new(
                    "provided url had a `title_no` query, but nothing was after the `=`",
                ));
            }

            // TODO: When if-let arm guards(https://github.com/rust-lang/rust/issues/51114) becomes stable might be able
            // clean this up more.
            Some(("title_no", id)) if id.chars().all(|ch| ch.is_ascii_digit()) => id
                .parse::<u32>()
                .map_err(|_| InvalidWebtoonUrl::new("provided `weboons.com` Webtoon homepage url had a valid `title_no=N` query, but the value was too large to fit in a `u32`"))?,

            Some(("title_no", _)) => return Err(InvalidWebtoonUrl::new("provided url had a `title_no` query, but the value was not a valid digit")),

            Some(_) => {
                return Err(InvalidWebtoonUrl::new(
                    "provided `webtoons.com` Webtoon homepage url did not have a `title_no` query",
                ));
            }

            None => {
                return Err(InvalidWebtoonUrl::new(
                    "`title_no` should always have a `=` separator",
                ));
            }
        };

        let webtoon = Webtoon {
            client: client.clone(),
            language,
            scope,
            slug: slug.into(),
            id,
            page: Arc::new(RwLock::new(None)),
        };

        Ok(webtoon)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn should_make_webtoon_from_url() {
        let webtoon = Webtoon::from_url_with_client(
            "https://www.webtoons.com/en/fantasy/tower-of-god/list?title_no=95",
            &Client::new(),
        )
        .unwrap();

        assert_eq!(webtoon.language, Language::En);
        assert_eq!(webtoon.scope, Scope::Original(Genre::Fantasy));
        assert_eq!(webtoon.slug.as_ref(), "tower-of-god");
        assert_eq!(webtoon.id, 95);
    }
}
