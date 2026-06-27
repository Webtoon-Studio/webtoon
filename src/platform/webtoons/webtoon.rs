//! A Webtoon on `webtoons.com` and its associated operations.

mod homepage;

pub mod episode;
pub mod post;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "rss")]
pub mod rss;
#[cfg(feature = "rss")]
use rss::Rss;

use super::error::WebtoonError;
use super::originals::Schedule;
use super::{Client, creator::Creator};
use crate::platform::webtoons::dashboard;
use crate::platform::webtoons::webtoon::episode::Episode;
use crate::platform::webtoons::webtoon::homepage::Homepage;
use crate::stdx::error::{assume_matches, assumption};
use crate::{
    platform::webtoons::{
        error::{
            ClientError, EpisodesError, InvalidWebtoonUrl, PostsError, SessionError,
            SubscribersError, ViewsError,
        },
        webtoon::post::Comment,
    },
    stdx::{
        cache::{Cache, Store},
        http::IRetry,
    },
};
use core::fmt::{self, Debug};
use std::str::FromStr;
use std::sync::Arc;

/// A Webtoon from `webtoons.com`.
///
/// Obtained through a [`Client`] via [`Client::webtoon()`], [`Client::webtoon_from_url()`] or
/// [`Creator::webtoons()`];
/// not constructed directly. Abstracts over both [`Type::Original`] and [`Type::Canvas`] webtoons.
///
/// See the method documentation for available operations.
#[derive(Clone)]
pub struct Webtoon {
    pub(super) client: Client,
    pub(super) id: u32,
    // Some genre for an original or `canvas` for canvas webtoons: "fantasy" or "canvas"
    pub(super) scope: Scope,
    /// URL slug of the Webtoon name: Tower of God -> tower-of-god
    pub(super) slug: Arc<str>,
    /// Cache for data on the Webtoon homepage: title, summary, etc.
    pub(super) homepage: Cache<Homepage>,
}

impl Debug for Webtoon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            client: _,
            id,
            scope,
            slug,
            homepage: page,
        } = self;

        f.debug_struct("Webtoon")
            .field("id", id)
            .field("scope", scope)
            .field("slug", slug)
            .field("page", page)
            .finish()
    }
}

impl Eq for Webtoon {}
impl PartialEq for Webtoon {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Webtoon {
    /// Returns the id of this `Webtoon`.
    ///
    /// This corresponds to the `title_no` query parameter: `https://www.webtoons.com/en/fantasy/osora/list?title_no=6202`
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6202, Type::Original).await?.expect("`6202` exists");
    ///
    /// assert_eq!(6202, webtoon.id());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn id(&self) -> u32 {
        let webtoon = self;
        webtoon.id
    }

    /// Returns the type of this `Webtoon`: [`Original`](variant@Type::Original) or [`Canvas`](variant@Type::Canvas).
    ///
    /// For doing simple boolean checks, prefer [`is_canvas()`](Webtoon::is_canvas()) or [`is_original()`](Webtoon::is_original()).
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
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
    #[must_use]
    pub fn r#type(&self) -> Type {
        let webtoon = self;
        match webtoon.scope {
            Scope::Original(_) => Type::Original,
            Scope::Canvas => Type::Canvas,
        }
    }

    /// Returns if `Webtoon` is an [`Original`](variant@Type::Original) type.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6880, Type::Original).await?.expect("`6880` exists");
    ///
    /// assert!(webtoon.is_original());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_original(&self) -> bool {
        let webtoon = self;
        matches!(webtoon.r#type(), Type::Original)
    }

    /// Returns if `Webtoon` is a [`Canvas`](variant@Type::Canvas) type.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(991073, Type::Canvas).await?.expect("`991073` exists");
    ///
    /// assert!(webtoon.is_canvas());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_canvas(&self) -> bool {
        let webtoon = self;
        matches!(webtoon.r#type(), Type::Canvas)
    }

    /// Returns the title of this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6880, Type::Original).await?.expect("`6880` exists");
    ///
    /// assert_eq!("I Became a Level 999 Demon Queen", webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn title(&self) -> Result<String, WebtoonError> {
        let webtoon = self;
        match webtoon.homepage.get() {
            Store::Value(page) => Ok(page.title().to_owned()),
            Store::Empty => {
                let homepage = homepage::scrape(webtoon).await?;
                let title = homepage.title().to_owned();
                webtoon.homepage.insert(homepage);
                Ok(title)
            }
        }
    }

    /// Returns the [`Creator`]'s' for this `Webtoon`.
    ///
    /// `Canvas` webtoon's always have exactly one creator.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(794611, Type::Canvas).await?.expect("`794611` exists");
    ///
    /// let creators = webtoon.creators().await?;
    ///
    /// assert_matches!(creators.as_slice(), [creator] if creator.username() == "AlmightyConurbano");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn creators(&self) -> Result<Vec<Creator>, WebtoonError> {
        let webtoon = self;
        match webtoon.homepage.get() {
            Store::Value(page) => Ok(page.creators().to_vec()),
            Store::Empty => {
                let page = homepage::scrape(webtoon).await?;
                let creators = page.creators().to_vec();
                webtoon.homepage.insert(page);
                Ok(creators)
            }
        }
    }

    /// Returns the [`Genre`]'s for this [`Webtoon`].
    ///
    /// [`Original`](variant@Type::Original) webtoons have exactly one; [`Canvas`](variant@Type::Canvas) webtoons have one or two.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, webtoon::Genre, Client};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1039653, Type::Canvas).await?.expect("`1039653` exists");
    ///
    /// let genres = webtoon.genres().await?;
    ///
    /// assert_matches!(genres.as_slice(), [action, fantasy] if action == &Genre::Action && fantasy == &Genre::Fantasy);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn genres(&self) -> Result<Vec<Genre>, WebtoonError> {
        let webtoon = self;
        match webtoon.homepage.get() {
            Store::Value(page) => Ok(page.genres().to_vec()),
            Store::Empty => {
                let page = homepage::scrape(webtoon).await?;
                let genres = page.genres().to_vec();
                webtoon.homepage.insert(page);
                Ok(genres)
            }
        }
    }

    /// Returns the summary for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(95, Type::Original).await?.expect("`95` exists");
    ///
    /// assert_eq!("What do you desire? Money and wealth? Honor and pride? Authority and power? Revenge? Or something that transcends them all? Whatever you desire—it's here.", webtoon.summary().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn summary(&self) -> Result<String, WebtoonError> {
        let webtoon = self;
        match webtoon.homepage.get() {
            Store::Value(page) => Ok(page.summary().to_owned()),
            Store::Empty => {
                let page = homepage::scrape(webtoon).await?;
                let summary = page.summary().to_owned();
                webtoon.homepage.insert(page);
                Ok(summary)
            }
        }
    }

    /// Returns the total number of views for this [`Webtoon`].
    ///
    /// For `Canvas` webtoons where [`Client::with_session()`] was used and the session belongs
    /// to the webtoon's creator, the total is summed from the analytics dashboard. Otherwise, the
    /// value is scraped from the webtoon's homepage and may be truncated.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1320, Type::Original).await?.expect("`1320` exists");
    ///
    /// // `Original` webtoons always use the homepage value, which may be truncated.
    /// assert_eq!(1_400_000_000, webtoon.views().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn views(&self) -> Result<u64, ViewsError> {
        let webtoon = self;
        let client = &self.client;

        match webtoon.r#type() {
            Type::Canvas
                if let session = client.session.validate(client).await
                    // A missing or invalid session falls back to the homepage scrape below.
                    && !matches!(session, Err(SessionError::NoSessionProvided | SessionError::InvalidSession))
                    && let valid_session = session?
                    && let user = client.fetch_webtoon_user_info(valid_session, webtoon).await?
                    && user.is_webtoon_creator() =>
            {
                let episodes = dashboard::analytics::episodes(webtoon).await?;

                let views = episodes
                    .iter()
                    .filter_map(|episode| episode.views.map(u64::from))
                    .sum::<u64>();

                Ok(views)
            }
            Type::Original | Type::Canvas => match webtoon.homepage.get() {
                Store::Value(homepage) => Ok(homepage.views()),
                Store::Empty => {
                    let homepage = homepage::scrape(webtoon).await?;
                    let views = homepage.views();
                    webtoon.homepage.insert(homepage);
                    Ok(views)
                }
            },
        }
    }

    /// Returns the total number of subscribers for this [`Webtoon`].
    ///
    /// For `Canvas` webtoons where [`Client::with_session()`] was used and the session belongs
    /// to the webtoon's creator, the count is retrieved from the creator's stats dashboard.
    /// Otherwise, the value is scraped from the webtoon's homepage and may be rounded.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1436, Type::Original).await?.expect("`1436` exists");
    ///
    /// // `Original` webtoons always use the homepage value, which may be truncated.
    /// assert_eq!(7_500_000, webtoon.subscribers().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn subscribers(&self) -> Result<u32, SubscribersError> {
        let webtoon = self;
        let client = &self.client;

        match webtoon.r#type() {
            Type::Canvas
                if let session = client.session.validate(client).await
                    // A missing or invalid session falls back to the homepage scrape below.
                    && !matches!(session, Err(SessionError::NoSessionProvided | SessionError::InvalidSession))
                    && let valid_session = session?
                    && let user = client.fetch_webtoon_user_info(valid_session, webtoon).await?
                    && user.is_webtoon_creator()
                    // HACK: There is the new analytics page that has subscribers
                    // but in looking, it doesn't really seem to match the displayed
                    // subscriber count on the homepage; not really sure what to do?
                    && false =>
            {
                todo!()
            }
            Type::Original | Type::Canvas => match webtoon.homepage.get() {
                Store::Value(homepage) => Ok(homepage.subscribers()),
                Store::Empty => {
                    let homepage = homepage::scrape(webtoon).await?;
                    let subscribers = homepage.subscribers();
                    webtoon.homepage.insert(homepage);
                    Ok(subscribers)
                }
            },
        }
    }

    /// Returns the thumbnail URL for this [`Webtoon`], if any.
    ///
    /// [`Original`](variant@Type::Original) webtoons always return `None`, as `webtoons.com`
    /// no longer exposes thumbnails for them.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(19985, Type::Canvas).await?.expect("`19985` exists");
    /// assert_matches!(webtoon.thumbnail().await?, Some(url) if url.starts_with("https://swebtoon-phinf.pstatic.net"));
    ///
    /// let webtoon = client.webtoon(4937, Type::Original).await?.expect("`4937` exists");
    /// assert_matches!(webtoon.thumbnail().await?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn thumbnail(&self) -> Result<Option<String>, WebtoonError> {
        let webtoon = self;
        match webtoon.r#type() {
            // WHY: `webtoons.com` removed Originals' thumbnails on their homepages.
            Type::Original => Ok(None),
            Type::Canvas => match webtoon.homepage.get() {
                Store::Value(homepage) => Ok(homepage.thumbnail().map(ToOwned::to_owned)),
                Store::Empty => {
                    let homepage = homepage::scrape(webtoon).await?;
                    let thumbnail = homepage.thumbnail().map(ToOwned::to_owned);
                    webtoon.homepage.insert(homepage);
                    Ok(thumbnail)
                }
            },
        }
    }

    /// Returns the release schedule for this [`Webtoon`], if any.
    ///
    /// `Canvas` webtoons have no release schedule and always return `None`. `Original`
    /// webtoons return `Some(Schedule)` with the episode drop cadence.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Type, originals::{Schedule, Weekday}, error::Error};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(843910, Type::Canvas).await?.expect("`843910` exists");
    /// assert_matches!(webtoon.schedule().await?, None);
    ///
    /// let webtoon = client.webtoon(10201, Type::Original).await?.expect("`10201` exists");
    /// assert_matches!(webtoon.schedule().await?, Some(Schedule::Weekday(Weekday::Thursday)));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn schedule(&self) -> Result<Option<Schedule>, WebtoonError> {
        let webtoon = self;
        match webtoon.r#type() {
            // WHY: Canvas Webtoons do not have any schedule section on their homepages.
            Type::Canvas => Ok(None),
            Type::Original => match webtoon.homepage.get() {
                Store::Value(page) => Ok(page.schedule().cloned()),
                Store::Empty => {
                    let page = homepage::scrape(webtoon).await?;
                    let release = page.schedule().cloned();
                    webtoon.homepage.insert(page);
                    Ok(release)
                }
            },
        }
    }

    /// Returns `true` if this [`Webtoon`] has completed its run.
    ///
    /// `Canvas` webtoons have no completion state and always return `false` without failing.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(93, Type::Original).await?.expect("`95` exists");
    /// assert!(webtoon.is_completed().await?);
    ///
    /// let webtoon = client.webtoon(1132214, Type::Canvas).await?.expect("`1132214` exists");
    /// assert!(!webtoon.is_completed().await.expect("call cannot fail for Canvas Webtoons"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn is_completed(&self) -> Result<bool, WebtoonError> {
        let webtoon = self;
        match webtoon.r#type() {
            // WHY: Canvas stories don't have the ability to indicate whether they are completed or not.
            Type::Canvas => Ok(false),
            Type::Original => match webtoon.homepage.get() {
                Store::Value(homepage) => {
                    Ok(matches!(homepage.schedule(), Some(Schedule::Completed)))
                }
                Store::Empty => {
                    let homepage = homepage::scrape(webtoon).await?;
                    let is_completed = matches!(homepage.schedule(), Some(Schedule::Completed));
                    webtoon.homepage.insert(homepage);
                    Ok(is_completed)
                }
            },
        }
    }

    /// Returns the banner image URL for this [`Webtoon`], if any.
    ///
    /// `Canvas` webtoons have no banner image and always return `None` without failing.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(3349, Type::Original).await?.expect("`3349` exists");
    /// assert_matches!(webtoon.banner().await?, Some(url) if url.starts_with("https://swebtoon-phinf.pstatic.net"));
    ///
    /// let webtoon = client.webtoon(1064573, Type::Canvas).await?.expect("`1064573` exists");
    /// assert_matches!(webtoon.banner().await.expect("call cannot fail for Canvas Webtoons"), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn banner(&self) -> Result<Option<String>, WebtoonError> {
        let webtoon = self;
        match webtoon.r#type() {
            // WHY: Canvas Webtoons do not have any Banner available to them.
            Type::Canvas => Ok(None),
            Type::Original => match webtoon.homepage.get() {
                Store::Value(homepage) => Ok(homepage.banner().map(ToOwned::to_owned)),
                Store::Empty => {
                    let homepage = homepage::scrape(webtoon).await?;
                    let release = homepage.banner().map(ToOwned::to_owned);
                    webtoon.homepage.insert(homepage);
                    Ok(release)
                }
            },
        }
    }

    /// Returns all episodes for this [`Webtoon`].
    ///
    /// There is no guarantee to order.
    ///
    /// If [`Client::with_session()`] was used and the session belongs to the webtoon's creator,
    /// episodes are scraped from the dashboard and include drafts, paywalled episodes, view counts
    /// via [`Episode::views()`], and exact publication timestamps via [`Episode::published()`].
    ///
    /// Otherwise, only publicly visible episodes are returned and [`Episode::views()`] will return
    /// `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1018, Type::Original).await?.expect("`1018` exists");
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// assert_eq!(25, episodes.len());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn episodes(&self) -> Result<Vec<Episode>, EpisodesError> {
        let webtoon = self;
        let client = &self.client;

        match webtoon.r#type() {
            Type::Canvas
                if let session = client.session.validate(client).await
                    // A missing or invalid session falls back to the homepage scrape below.
                    && !matches!(session, Err(SessionError::NoSessionProvided | SessionError::InvalidSession))
                    && let valid_session = session?
                    && let user = client.fetch_webtoon_user_info(valid_session, webtoon).await?
                    && user.is_webtoon_creator() =>
            {
                Ok(dashboard::analytics::episodes(webtoon).await?)
            }
            Type::Original | Type::Canvas => Ok(homepage::episodes(webtoon).await?),
        }
    }

    /// Returns the [`Episode`] at `number` for this [`Webtoon`], if it exists.
    ///
    /// Unlike [`Webtoon::episodes()`], even without a session this includes episodes that are unpublished, paywalled,
    /// or no longer publicly visible. [`Episode::views()`] and [`Episode::published()`] will
    /// always return `None`; use [`Webtoon::episodes()`] with a valid session if you need that data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Type, error::{Error, EpisodeError}};
    /// # use std::assert_matches;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(95, Type::Original).await?.expect("`95` exists");
    ///
    /// // Viewable episode.
    /// assert_matches!(webtoon.episode(222).await?, Some(episode) if matches!(episode.title().await?.as_str(), "[Season 2] Ep. 141"));
    ///
    /// // Known hidden episode: exists but is not viewable.
    /// let episode = webtoon.episode(221).await?.expect("`221` exists");
    /// assert_matches!(episode.title().await, Err(EpisodeError::NotViewable));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn episode(&self, number: u16) -> Result<Option<Episode>, WebtoonError> {
        let webtoon = self;

        let episode = Episode::new(webtoon, number);

        if !episode.exists().await? {
            return Ok(None);
        }

        Ok(Some(episode))
    }

    /// Returns the first publicly published [`Episode`] of this [`Webtoon`].
    ///
    /// Unlike [`Webtoon::episodes()`], this does not traverse additional pages, and unlike
    /// [`Webtoon::episode()`], it includes episode metadata such as the publication date.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(4176, Type::Original).await?.expect("`4176` exists");
    ///
    /// let episode = webtoon.first_episode().await?;
    ///
    /// assert_eq!("Ep. 1 - The Return (1)", episode.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn first_episode(&self) -> Result<Episode, WebtoonError> {
        let webtoon = self;
        homepage::first_episode(webtoon).await
    }

    /// Returns a random publicly published [`Episode`] of this [`Webtoon`].
    ///
    /// Unlike [`Webtoon::episodes()`], this does not traverse additional pages, and unlike
    /// [`Webtoon::episode()`], it includes episode metadata such as the publication date.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(487522, Type::Canvas).await?.expect("`487522` exists");
    ///
    /// let episode = webtoon.random_episode().await?;
    ///
    /// assert!(episode.number() > 0);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn random_episode(&self) -> Result<Episode, WebtoonError> {
        let webtoon = self;
        homepage::random_episode(webtoon).await
    }

    /// Returns the total number of likes across all episodes of this [`Webtoon`].
    ///
    /// Includes all episodes regardless of visibility - public, paywalled, deleted, or
    /// unpublished - which may cause this total to differ from what is publicly displayed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(7666, Type::Original).await?.expect("`7666` exists");
    ///
    /// let likes = webtoon.likes().await?;
    ///
    /// assert!(likes > 0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn likes(&self) -> Result<u32, WebtoonError> {
        let webtoon = self;

        let mut episodes = 1..;

        let mut likes = 0;

        while let Some(number) = episodes.next()
            && let Some(episode) = webtoon.episode(number).await?
        {
            likes += episode.likes().await?;
        }

        Ok(likes)
    }

    // TODO: Turn into interator
    /// Returns all top-level comments across every episode of this [`Webtoon`].
    ///
    /// Deleted comments that still have replies are included; deleted comments
    /// without replies are not.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(886472, Type::Canvas).await?.expect("`886472` exists");
    ///
    /// for post in webtoon.posts().await? {
    ///    println!("{} left a post on {}!", post.poster().username(), webtoon.title().await?);
    ///    # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # }
    /// ```
    pub async fn posts(&self) -> Result<Vec<Comment>, PostsError> {
        let webtoon = self;

        let mut episodes = 1..;

        let mut posts = Vec::with_capacity(100);

        while let Some(number) = episodes.next()
            && let Some(episode) = webtoon.episode(number).await?
        {
            let mut comments = episode.posts();
            while let Some(comment) = comments.next().await? {
                posts.push(comment);
            }
        }

        Ok(posts)
    }

    /// Returns the RSS feed for this [`Webtoon`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(7666, Type::Original).await?.expect("`7666` exists");
    ///
    /// let rss = webtoon.rss().await?;
    ///
    /// assert_eq!("The White Tower’s Rogue Mage", rss.title());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg(feature = "rss")]
    pub async fn rss(&self) -> Result<Rss, WebtoonError> {
        let webtoon = self;
        rss::feed(webtoon).await
    }
}

impl Webtoon {
    #[inline]
    pub(super) async fn new_with_client(
        id: u32,
        r#type: Type,
        client: &Client,
    ) -> Result<Option<Self>, ClientError> {
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

        let (scope, slug, _) = match parse_url(url) {
            Ok(parts) => parts,
            Err(InvalidWebtoonUrl::UnsupportedLanguage) => {
                return Err(ClientError::UnsupportedLanguage);
            }
            Err(err) => {
                assumption!("`webtoons.com` returned an unexpected url format for `{url}`: {err}")
            }
        };

        let webtoon = Self {
            client: client.clone(),
            id,
            scope,
            slug: Arc::from(slug),
            homepage: Cache::empty(),
        };

        Ok(Some(webtoon))
    }

    #[inline]
    pub(super) fn from_url_with_client(
        url: &str,
        client: &Client,
    ) -> Result<Self, InvalidWebtoonUrl> {
        let Ok(url) = url::Url::parse(url) else {
            return Err(InvalidWebtoonUrl::Malformed {
                url: url.to_owned(),
                reason: String::from("invalid url schema"),
            });
        };

        let (scope, slug, id) = parse_url(&url)?;

        let webtoon = Self {
            client: client.clone(),
            scope,
            slug: slug.into(),
            id,
            homepage: Cache::empty(),
        };

        Ok(webtoon)
    }
}

#[inline]
fn parse_url(url: &url::Url) -> Result<(Scope, String, u32), InvalidWebtoonUrl> {
    let Some(mut segments) = url.path_segments() else {
        return Err(InvalidWebtoonUrl::Malformed {
            url: url.to_string(),
            reason: String::from(
                "a `webtoons.com` Webtoon url should have path segments (`/`); this url did not",
            ),
        });
    };

    assume_matches!(
        segments.next(),
        Some("en"),
        InvalidWebtoonUrl::UnsupportedLanguage
    );

    let Some(scope) = segments.next() else {
        return Err(InvalidWebtoonUrl::Malformed {
            url: url.to_string(),
            reason: String::from(
                "provided url didn't have a second segment representing the scope, e.g. `canvas`, `fantasy`",
            ),
        });
    };

    let scope = Scope::from_str(scope).map_err(|err| InvalidWebtoonUrl::Malformed {
        url: url.to_string(),
        reason: format!("found an unexpected scope in provided `webtoons.com` url: {err}"),
    })?;

    let slug = segments
            .next()
            .ok_or_else(|| InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: String::from("provided url didn't have a third segment representing the slug, e.g. `tower-of-god`"),
            })?
            .to_string();

    let id = url.query().ok_or_else(|| InvalidWebtoonUrl::Malformed {
        url: url.to_string(),
        reason: String::from(
            "a valid `webtoons.com` Webtoon url should have a `title_no` query, but none was found",
        ),
    })?;

    let id = match id.split_once('=') {
        Some(("title_no", "")) => {
            return Err(InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: String::from(
                    "provided url had a `title_no` query but nothing after the `=`",
                ),
            });
        }
        Some(("title_no", id)) if id.chars().all(|ch| ch.is_ascii_digit()) => id
            .parse::<u32>()
            .map_err(|err| InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: format!("the `title_no` value was too large to fit in a `u32`: {err}"),
            })?,
        Some(("title_no", _)) => {
            return Err(InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: String::from(
                    "provided url had a `title_no` query but the value was not a valid digit",
                ),
            });
        }
        Some(_) => {
            return Err(InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: String::from("provided url did not have a `title_no` query"),
            });
        }
        None => {
            return Err(InvalidWebtoonUrl::Malformed {
                url: url.to_string(),
                reason: String::from("`title_no` should always have a `=` separator"),
            });
        }
    };

    Ok((scope, slug, id))
}

/// Represents the type a Webtoon can be on `webtoons.com`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// An Original Webtoon.
    #[serde(alias = "WEBTOON")]
    Original,
    /// A Canvas Webtoon.
    #[serde(alias = "CHALLENGE")]
    Canvas,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Scope {
    Original(Genre),
    Canvas,
}

impl Scope {
    pub(super) fn as_slug(&self) -> &str {
        match self {
            Self::Canvas => "canvas",
            Self::Original(genre) => genre.as_slug(),
        }
    }
}

impl FromStr for Scope {
    type Err = ParseGenreError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let scope = match s {
            "canvas" => Self::Canvas,
            slug => Self::Original(Genre::from_str(slug)?),
        };

        Ok(scope)
    }
}

/// Represents a genre on `webtoons.com`.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub enum Genre {
    Comedy,
    Fantasy,
    Romance,
    SliceOfLife,
    SciFi,
    Drama,
    ShortStory,
    Action,
    Superhero,
    Heartwarming,
    Thriller,
    Horror,
    PostApocalyptic,
    Zombies,
    School,
    Supernatural,
    Animals,
    Mystery,
    Historical,
    /// Tiptoon
    Informative,
    Sports,
    Inspirational,
    AllAges,
    LGBTQ,
    RomanticFantasy,
    MartialArts,
    WesternPalace,
    EasternPalace,
    MatureRomance,
    /// Reincarnation/Time-travel
    TimeSlip,
    Local,
    /// Modern/Workplace
    CityOffice,
    Adaptation,
    Shonen,
    WebNovel,
    GraphicNovel,
}

impl Genre {
    /// Converts a [`Genre`] into a URL safe slug.
    ///
    /// Example:
    /// - `Genre::Action => "action"`,
    /// - `Genre::AllAges => "all-ages"`,
    #[inline]
    #[must_use]
    pub const fn as_slug(&self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::AllAges => "all-ages",
            Self::Animals => "animals",
            Self::Comedy => "comedy",
            Self::Drama => "drama",
            Self::Fantasy => "fantasy",
            Self::Heartwarming => "heartwarming",
            Self::Historical => "historical",
            Self::Horror => "horror",
            Self::Informative => "tiptoon",
            Self::Inspirational => "inspirational",
            Self::Mystery => "mystery",
            Self::PostApocalyptic => "post-apocalyptic",
            Self::Romance => "romance",
            Self::School => "school",
            Self::SciFi => "sf",
            Self::ShortStory => "short-story",
            Self::SliceOfLife => "slice-of-life",
            Self::Sports => "sports",
            Self::Superhero => "super-hero",
            Self::Supernatural => "supernatural",
            Self::Thriller => "thriller",
            Self::Zombies => "zombies",
            Self::LGBTQ => "bl-gl",
            Self::RomanticFantasy => "romantic-fantasy",
            Self::MartialArts => "martial-arts",
            Self::WesternPalace => "western-palace",
            Self::EasternPalace => "eastern-palace",
            Self::MatureRomance => "romance-m",
            Self::TimeSlip => "time-slip",
            Self::Local => "local",
            Self::CityOffice => "city-office",
            Self::Adaptation => "adaptation",
            Self::Shonen => "shonen",
            Self::WebNovel => "web-novel",
            Self::GraphicNovel => "graphic-novel",
        }
    }
}

/// An error that can happen when parsing a string into a [`Genre`].
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a known genre")]
pub struct ParseGenreError(String);

impl FromStr for Genre {
    type Err = ParseGenreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "COMEDY" | "Comedy" | "comedy" => Ok(Self::Comedy),
            "FANTASY" | "Fantasy" | "fantasy" => Ok(Self::Fantasy),
            "ROMANCE" | "Romance" | "romance" => Ok(Self::Romance),
            "SLICE OF LIFE" | "Slice of life" | "slice-of-life" => Ok(Self::SliceOfLife),
            "SCI-FI" | "Sci-fi" | "Sci-Fi" | "sf" | "SF" => Ok(Self::SciFi),
            "DRAMA" | "Drama" | "drama" => Ok(Self::Drama),
            "SHORT STORY" | "Short story" => Ok(Self::ShortStory),
            "ACTION" | "Action" | "action" => Ok(Self::Action),
            "ALL AGES" | "All Ages" => Ok(Self::AllAges),
            "SUPERHERO" | "Superhero" | "super-hero" | "superhero" => Ok(Self::Superhero),
            "HEARTWARMING" | "Heartwarming" | "heartwarming" => Ok(Self::Heartwarming),
            "THRILLER" | "Thriller" | "thriller" => Ok(Self::Thriller),
            "HORROR" | "Horror" | "horror" => Ok(Self::Horror),
            "POST APOCALYPTIC" | "Post apocalyptic" | "Post-apocalyptic" => {
                Ok(Self::PostApocalyptic)
            }
            "ZOMBIES" | "Zombies" | "zombies" => Ok(Self::Zombies),
            "SCHOOL" | "School" | "school" => Ok(Self::School),
            "SUPERNATURAL" | "Supernatural" | "supernatural" => Ok(Self::Supernatural),
            "ANIMALS" | "Animals" | "animals" => Ok(Self::Animals),
            "CRIME/MYSTERY" | "Crime/Mystery" | "Mystery" | "mystery" => Ok(Self::Mystery),
            "HISTORICAL" | "Historical" | "historical" => Ok(Self::Historical),
            "INFORMATIVE" | "Informative" | "informative" | "tiptoon" => Ok(Self::Informative),
            "SPORTS" | "Sports" | "sports" => Ok(Self::Sports),
            "INSPIRATIONAL" | "Inspirational" | "inspirational" => Ok(Self::Inspirational),
            "LGBTQ+ / Y" | "LGBTQ+" | "bl-gl" | "LGBTQI+" => Ok(Self::LGBTQ),
            "romantic-fantasy" | "Romance Fantasy" | "ROMANTIC_FANTASY" => {
                Ok(Self::RomanticFantasy)
            }
            "martial-arts" => Ok(Self::MartialArts),
            "western-palace" => Ok(Self::WesternPalace),
            "eastern-palace" => Ok(Self::EasternPalace),
            "romance-m" => Ok(Self::MatureRomance),
            "time-slip" => Ok(Self::TimeSlip),
            "local" | "LOCAL" => Ok(Self::Local),
            "city-office" => Ok(Self::CityOffice),
            "adaptation" => Ok(Self::Adaptation),
            "shonen" => Ok(Self::Shonen),
            "web-novel" | "WEBNOVEL" => Ok(Self::WebNovel),
            "graphic-novel" | "GRAPHIC_NOVEL" | "Graphic Novel" => Ok(Self::GraphicNovel),
            _ => Err(ParseGenreError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn should_make_webtoon_from_url() {
        let webtoon = Webtoon::from_url_with_client(
            "https://www.webtoons.com/en/fantasy/tower-of-god/list?title_no=95",
            &Client::new(),
        )
        .unwrap();

        assert_eq!(webtoon.scope, Scope::Original(Genre::Fantasy));
        assert_eq!(webtoon.slug.as_ref(), "tower-of-god");
        assert_eq!(webtoon.id, 95);
    }

    #[test]
    fn should_parse_genres_from_str() -> Result<(), Box<dyn std::error::Error>> {
        let genre = Genre::from_str("Slice of life")?;
        assert_eq!(Genre::SliceOfLife, genre);
        Ok(())
    }
}
