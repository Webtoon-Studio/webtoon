//! Module containing things related to an episode on `webtoons.com`.

use super::{Webtoon, post::PinRepresentation};
use crate::platform::webtoons::webtoon::post::{Comments, id::Id};
use crate::stdx::cache::{Cache, Store};
use crate::{
    platform::webtoons::{
        dashboard::episodes::DashboardStatus,
        error::{EpisodeError, LikesError, PostsError},
    },
    stdx::time::DateOrDateTime,
};
use arrayvec::ArrayVec;
use assumptions::{Assume, Assumption, assume, assume_matches, assumption};
use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::hash::Hash;
use url::Url;

use std::debug_assert as ensure;

/// An episode on `webtoons.com`.
///
/// Obtained via [`Webtoon::episodes()`] or [`Webtoon::episode()`]; not constructed directly.
///
/// Any `Episode` instance is guaranteed to exist on the platform.
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{error::Error, Client, Type};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let webtoon = client.webtoon(2960, Type::Original).await?.expect("`2960` exists");
///
/// if let Some(episode) = webtoon.episode(187).await? {
///     assert_eq!("(S2) Ep. 187 - Gods Plan", episode.title().await?);
///     # return Ok(());
/// }
/// # unreachable!("should have entered the episode block and returned");
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Episode {
    pub(crate) webtoon: Webtoon,
    pub(crate) number: u16,
    pub(crate) title: Cache<String>,
    pub(crate) published: Option<Published>,
    pub(crate) length: Cache<Option<u32>>,
    // TODO: Add super likes.
    pub(crate) views: Option<u32>,
    pub(crate) thumbnail: Cache<Url>,
    pub(crate) note: Cache<Option<String>>,
    pub(crate) ad_status: Option<AdStatus>,
    pub(crate) published_status: Option<PublishedStatus>,
    pub(crate) panels: Cache<Vec<Panel>>,
    pub(crate) top_comments: Cache<ArrayVec<Id, 3>>,
}

impl std::fmt::Debug for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            webtoon: _,
            number,
            title,
            published,
            length,
            views,
            thumbnail,
            note,
            ad_status,
            published_status,
            panels,
            top_comments,
        } = self;

        f.debug_struct("Episode")
            .field("number", number)
            .field("title", title)
            .field("published", published)
            .field("length", length)
            .field("views", views)
            .field("thumbnail", thumbnail)
            .field("note", note)
            .field("ad_status", ad_status)
            .field("published_status", published_status)
            .field("panels", panels)
            .field("top_comments", top_comments)
            .finish()
    }
}

impl Episode {
    /// Returns the number of this [`Episode`].
    ///
    /// This matches up with the `episode_no=` URL query: [`episode_no=25`]
    ///
    /// This may differ from the displayed `#NUMBER` on the episode list if any episodes have
    /// been deleted, shifting the visible numbering.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(7370, Type::Original).await?.expect("`7370` exists");
    ///
    /// if let Some(episode) = webtoon.episode(25).await? {
    ///     assert_eq!(25, episode.number());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`episode_no=25`]: https://www.webtoons.com/en/fantasy/the-roguish-guard-in-a-medieval-fantasy/episode-25/viewer?title_no=7370&episode_no=25
    #[inline]
    #[must_use]
    pub fn number(&self) -> u16 {
        let episode = self;
        episode.number
    }

    /// Returns the title of this [`Episode`].
    ///
    /// Returns [`EpisodeError::NotViewable`] if the episode is hidden or deleted, which can
    /// occur when the episode was obtained via [`Webtoon::episode()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6532, Type::Original).await?.expect("`6532` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     assert_eq!("Ep. 1 - Prologue", episode.title().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn title(&self) -> Result<String, EpisodeError> {
        let episode = self;
        match episode.title.get() {
            Store::Value(title) => Ok(title),
            Store::Empty => {
                episode.scrape().await?;
                match episode.title.get() {
                    Store::Value(title) => Ok(title),
                    Store::Empty => {
                        unreachable!("`episode.title` should be populated after `episode.scrape()`")
                    }
                }
            }
        }
    }

    /// Returns the season number for this [`Episode`], if any.
    ///
    /// Inferred from the episode title by matching patterns like `[Season 2]`, `(Season 2)`,
    /// `[S2]`, or `(S2)`. Returns `None` if no pattern is found, or
    /// [`EpisodeError::NotViewable`] if the episode is hidden or deleted.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(95, Type::Original).await?.expect("`95` exists");
    ///
    /// if let Some(episode) = webtoon.episode(652).await? {
    ///     assert_eq!(Some(3), episode.season().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn season(&self) -> Result<Option<u8>, EpisodeError> {
        let episode = self;
        let title = episode.title().await?;
        Ok(season(&title))
    }

    /// Returns the creator's note for this [`Episode`], if any.
    ///
    /// Returns `None` if no note is found, or [`EpisodeError::NotViewable`] if the
    /// episode is hidden or deleted.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(261984, Type::Canvas).await?.expect("`261984` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await?
    ///     && let Some(note) = episode.note().await? {
    ///         assert!(note.starts_with("Find me as Jayessart"));
    ///         # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn note(&self) -> Result<Option<String>, EpisodeError> {
        let episode = self;
        match episode.note.get() {
            Store::Value(note) => Ok(note),
            Store::Empty => {
                episode.scrape().await?;
                match episode.note.get() {
                    Store::Value(note) => Ok(note),
                    Store::Empty => {
                        unreachable!("`episode.note` should be populated after `episode.scrape()`")
                    }
                }
            }
        }
    }

    /// Returns the total vertical length of this [`Episode`] in pixels, if any.
    ///
    /// Returns `None` for episodes with audio or GIFs, as that viewer is unsupported.
    /// Returns [`EpisodeError::NotViewable`] for paywalled or app-only episodes.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     assert_eq!(Some(89600), episode.length().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn length(&self) -> Result<Option<u32>, EpisodeError> {
        let episode = self;
        match episode.length.get() {
            Store::Value(length) => Ok(length),
            Store::Empty => {
                episode.scrape().await?;
                match episode.length.get() {
                    Store::Value(length) => Ok(length),
                    Store::Empty => unreachable!(
                        "`episode.length` should be populated after `episode.scrape()`"
                    ),
                }
            }
        }
    }

    /// Returns the [`Published`] date or datetime for this [`Episode`], if any.
    ///
    /// Only populated when the episode was obtained via [`Webtoon::episodes()`] or
    /// [`Webtoon::rss()`]; always `None` for episodes from [`Webtoon::episode()`].
    ///
    /// For `Canvas` episodes fetched with a creator session, draft episodes return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// let mut episodes = webtoon.episodes().await?;
    ///
    /// episodes.sort_unstable_by_key(|episode| episode.number());
    ///
    /// if let Some(episode) = episodes.first()
    ///    && let Some(published) = episode.published()  {
    ///      assert_eq!(29, published.day());
    ///      assert_eq!(4, published.month());
    ///      assert_eq!(2025, published.year());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn published(&self) -> Option<Published> {
        let episode = self;
        episode.published
    }

    /// Returns the view count for this [`Episode`], if available.
    ///
    /// Only populated for `Canvas` episodes obtained via [`Webtoon::episodes()`] with a
    /// creator session. All other cases - `Original` webtoons, no
    /// session, or episodes from [`Webtoon::episode()`] - always return `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("my-session");
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// if let Some(episode) = episodes.first() {
    ///     println!("episode {} has {:?} views", episode.number(), episode.views());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn views(&self) -> Option<u32> {
        let episode = self;
        episode.views
    }

    /// Returns the like count for this [`Episode`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("episode {} has {} likes", episode.number(), episode.likes().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn likes(&self) -> Result<u32, LikesError> {
        let episode = self;
        let client = &self.webtoon.client;

        let response = client.fetch_episodes_likes(episode).await?;

        let contents = response
            .result
            .contents
            .first()
            .assumption("`contents` field in `webtoons.com` likes API response should have at least one element")?;

        let likes = contents
            .reactions
            .first()
            .map(|likes| likes.count)
            // NOTE: A like count starts at zero.
            //
            // Given that we have an `Episode` we know the episode is valid, and
            // thus, if the reactions count is empty, we can safely assume that
            // the there are no likes yet, and should just default to `0`.
            .unwrap_or_default();

        Ok(likes)
    }

    /// Returns the comment and reply counts for this [`Episode`] as `(comments, replies)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let (comments, replies) = episode.comments_and_replies().await?;
    ///     println!("episode {} has {comments} comments and {replies} replies", episode.number());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn comments_and_replies(&self) -> Result<(u32, u32), PostsError> {
        let episode = self;
        let client = &self.webtoon.client;

        let response = client
            .fetch_episode_posts(episode, None, 1, PinRepresentation::None)
            .await?;

        let comments = response.result.active_root_post_count;
        let replies = response.result.active_post_count - comments;

        Ok((comments, replies))
    }

    /// Returns an iterator over the top-level comments for this [`Episode`], newest first.
    ///
    /// Deleted comments with replies are included; deleted comments without replies are not.
    ///
    /// If a session was provided, each [`Comment`] will contain additional poster metadata,
    /// such as whether it was left by the session user.
    ///
    /// Due to API pagination behavior, duplicate comments may occasionally appear.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(1046567, Type::Canvas).await?.expect("`1046567` exists");
    ///
    /// let episode = webtoon.episode(1).await?.expect("episode 1 exists");
    ///
    /// let mut comments = episode.posts();
    ///
    /// while let Some(comment) = comments.next().await? {
    ///     println!("{} left a comment on episode {} of {}", comment.poster().username(), episode.number(), webtoon.title().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # }
    /// ```
    #[inline]
    pub fn posts(&self) -> Comments<'_> {
        let episode = self;
        Comments::new(episode)
    }

    /// Returns a list of panels for this [`Episode`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(843910, Type::Canvas).await?.expect("`843910` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for panel in episode.panels().await? {
    ///         println!("url: {}", panel.url());
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn panels(&self) -> Result<Vec<Panel>, EpisodeError> {
        let episode = self;
        match episode.panels.get() {
            Store::Value(panels) => Ok(panels),
            Store::Empty => {
                episode.scrape().await?;
                match episode.panels.get() {
                    Store::Value(panels) => Ok(panels),
                    Store::Empty => unreachable!(
                        "`episode.panels` should be populated after `episode.scrape()`"
                    ),
                }
            }
        }
    }

    /// Returns the thumbnail URL for this [`Episode`].
    ///
    /// Returns [`EpisodeError::NotViewable`] for app-only episodes.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6679, Type::Original).await?.expect("`6679` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("thumbnail: {}", episode.thumbnail().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn thumbnail(&self) -> Result<String, EpisodeError> {
        let episode = self;
        match episode.thumbnail.get() {
            Store::Value(thumbnail) => Ok(thumbnail.to_string()),
            Store::Empty => {
                episode.scrape().await?;
                match episode.thumbnail.get() {
                    Store::Value(thumbnail) => Ok(thumbnail.to_string()),
                    Store::Empty => unreachable!(
                        "`episode.thumbnail` should be populated after `episode.scrape()`"
                    ),
                }
            }
        }
    }

    /// Returns the [`PublishedStatus`] of this [`Episode`], if any.
    ///
    /// Only populated when the episode was obtained via [`Webtoon::episodes()`]; always `None`
    /// for episodes from [`Webtoon::episode()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, webtoon::episode::PublishedStatus, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6889, Type::Original).await?.expect("`6889` exists");
    ///
    /// let mut episodes = webtoon.episodes().await?;
    ///
    /// episodes.sort_unstable_by_key(|episode| episode.number());
    ///
    /// if let Some(episode) = episodes.first() {
    ///     assert_eq!(episode.number(), 1);
    ///     assert_eq!(episode.published_status(), Some(PublishedStatus::Published));
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn published_status(&self) -> Option<PublishedStatus> {
        let episode = self;
        episode.published_status
    }

    /// Returns the [`AdStatus`] of this [`Episode`], if any.
    ///
    /// Only available for `Canvas` episodes obtained via [`Webtoon::episodes()`] with a
    /// creator session. Always `None` for `Original` webtoons, missing
    /// or non-creator sessions, or episodes from [`Webtoon::episode()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, webtoon::episode::AdStatus, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let webtoon = client.webtoon(6679, Type::Canvas).await?.expect("`6679` exists");
    ///
    /// let mut episodes = webtoon.episodes().await?;
    ///
    /// episodes.sort_unstable_by_key(|episode| episode.number());
    ///
    /// if let Some(episode) = episodes.first() {
    ///     match episode.ad_status() {
    ///             Some(AdStatus::Yes) => println!("Episode is behind an ad."),
    ///             Some(AdStatus::No) => println!("Episode is no longer behind an ad."),
    ///             Some(AdStatus::Never) => println!("Episode was never behind an ad."),
    ///             None => unreachable!("must use `Webtoon::episodes()` and have valid session!"),
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn ad_status(&self) -> Option<AdStatus> {
        let episode = self;
        episode.ad_status
    }

    /// Returns `true` if this [`Episode`] has a [`PublishedStatus::Published`] status.
    ///
    /// Returns `false` if the status is unknown, a draft, or removed.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, webtoon::episode::PublishedStatus, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6889, Type::Original).await?.expect("`6889` exists");
    ///
    /// let mut episodes = webtoon.episodes().await?;
    ///
    /// episodes.sort_unstable_by_key(|episode| episode.number());
    ///
    /// if let Some(episode) = episodes.first() {
    ///     assert!(episode.is_published());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_published(&self) -> bool {
        let episode = self;
        matches!(episode.published_status, Some(PublishedStatus::Published))
    }

    /// Downloads the panels of this [`Episode`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6889, Type::Original).await?.expect("`6889` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let panels = episode.download().await?;
    ///     assert_eq!(201, panels.count());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "download")]
    #[expect(
        clippy::missing_panics_doc,
        reason = "a Semaphore not being able to close is an internal logic error, and caught in CI"
    )]
    pub async fn download(&self) -> Result<DownloadedPanels, EpisodeError> {
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let episode = self;
        let client = &self.webtoon.client;

        let mut panels = episode.panels().await?;

        // PERF: Download N panels at a time. Without this it will be a sequential.
        let semaphore = Arc::new(Semaphore::new(100));

        let mut tasks = Vec::with_capacity(panels.len());

        for mut panel in std::mem::take(&mut panels) {
            let semaphore = Arc::clone(&semaphore);
            let client = client.clone();
            tasks.push(tokio::spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("semaphore should not be closed during `Episode::download`");

                panel.download(&client).await?;

                Ok::<Panel, EpisodeError>(panel)
            }));
        }

        let mut height = 0;
        let mut width = 0;

        for task in tasks {
            let panel = task.await.expect("joining task should not panic")?;

            height += panel.height;
            // NOTE: Not all panels are guaranteed to be the same width. When it
            // comes to making building up a single image later on, this is needed
            // to get the max width of all panels and then just fit to that.
            width = width.max(panel.width);

            panels.push(panel);
        }

        ensure!(panels.is_sorted_by_key(|panel| panel.number));

        Ok(DownloadedPanels {
            panels,
            height,
            width,
        })
    }
}

impl Episode {
    pub(crate) fn new(webtoon: &Webtoon, number: u16) -> Self {
        Self {
            webtoon: webtoon.clone(),
            number,
            title: Cache::empty(),
            published: None,
            length: Cache::empty(),
            thumbnail: Cache::empty(),
            note: Cache::empty(),
            panels: Cache::empty(),
            views: None,
            ad_status: None,
            published_status: None,
            top_comments: Cache::empty(),
        }
    }

    async fn scrape(&self) -> Result<(), EpisodeError> {
        let episode = self;
        let client = &self.webtoon.client;

        let html = client
            .fetch_episode_page(&self.webtoon, self.number)
            .await?;

        if only_viewable_on_app(&html) {
            return Err(EpisodeError::NotViewable);
        }

        episode.title.insert(title(&html)?);
        episode.thumbnail.insert(thumbnail(&html, episode.number)?);
        episode.length.insert(length(&html)?);
        episode.note.insert(note(&html)?);
        episode.panels.insert(panels(&html, episode.number)?);

        Ok(())
    }

    pub(super) async fn exists(&self) -> Result<bool, reqwest::Error> {
        let episode = self;
        let client = &self.webtoon.client;
        client.check_if_episode_exists(episode).await
    }
}

pub(crate) fn season(title: &str) -> Option<u8> {
    let patterns = [
        r"\[Season (?P<season>\d+)\]",
        r"\[S(?P<season>\d+)\]",
        r"\(S(?P<season>\d+)\)",
        r"\(Season (?P<season>\d+)\)",
    ];

    for pattern in patterns {
        let regex = Regex::new(pattern).expect("all regex in `patterns` should be valid");
        if let Some(capture) = regex.captures(title) {
            let season = capture["season"]
                .parse::<u8>()
                .expect(r"regex matched `\d+` so should be parsable as `u8`");
            return Some(season);
        }
    }

    None
}

impl Hash for Episode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for Episode {}

/// The ad status of an [`Episode`].
///
/// Only available when the episode was obtained via [`Webtoon::episodes()`] with a creator session.
#[derive(Debug, Clone, Copy)]
pub enum AdStatus {
    /// Episode is currently behind an ad.
    Yes,
    /// Episode is not currently behind an ad.
    No,
    /// Episode was never behind an ad.
    Never,
}

/// The publication status of an [`Episode`].
///
/// Only available when the episode was obtained via [`Webtoon::episodes()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishedStatus {
    /// The episode is publicly available, including episodes behind ad or fast-pass paywalls.
    Published,
    /// The episode has not yet been published in any capacity.
    Draft,
    /// The episode was previously published but has since been removed.
    Removed,
}

impl From<DashboardStatus> for PublishedStatus {
    fn from(value: DashboardStatus) -> Self {
        match value {
            DashboardStatus::Published | DashboardStatus::AdOn | DashboardStatus::AdOff => {
                Self::Published
            }
            DashboardStatus::Draft
            | DashboardStatus::Approved
            | DashboardStatus::Ready
            | DashboardStatus::InReview
            | DashboardStatus::Disapproved
            | DashboardStatus::DisapprovedAuto => Self::Draft,
            DashboardStatus::Removed => Self::Removed,
        }
    }
}

fn title(html: &Html) -> Result<String, Assumption> {
    let selector = Selector::parse("div.subj_info>.subj_episode") //
        .expect("`div.subj_info>.subj_episode` should be a valid selector");

    let title = html
        .select(&selector)
        .next()
        .assumption("`.subj_episode` element should be present on `webtoons.com` episode page")?
        .text()
        .next()
        .assumption("`.subj_episode` element on `webtoons.com` episode page should contain text")?;

    assume!(
        !title.is_empty(),
        "`webtoons.com` episode title should never be empty"
    );

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn length(html: &Html) -> Result<Option<u32>, Assumption> {
    // NOTE:
    // Most panel pixels end in a `.0`, but this is not guaranteed. The values
    // also have the potential to be a whole number, with no `.`. This is true
    // for both height and width.

    if is_audio_reader(html) {
        return Ok(None);
    }

    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut length = 0;

    for img in html.select(&selector) {
        length += height(img)?;
    }

    Ok(Some(length))
}

fn note(html: &Html) -> Result<Option<String>, Assumption> {
    let selector = Selector::parse(r".creator_note>.author_text") //
        .expect("`.creator_note>.author_text` should be a valid selector");

    let Some(selection) = html.select(&selector).next() else {
        return Ok(None);
    };

    let note = selection
        .text()
        .next()
        .assumption("`.author_text` element on `webtoons.com` episode page should contain text")?;

    assume!(
        !note.is_empty(),
        "`.author_text` element on `webtoons.com` episode page should never be empty"
    );

    Ok(Some(html_escape::decode_html_entities(note).to_string()))
}

fn thumbnail(html: &Html, episode: u16) -> Result<Url, Assumption> {
    let selector =
        Selector::parse(r"div.episode_lst>div.episode_cont>ul>li") //
            .expect(r"`div.episode_lst>div.episode_cont>ul>li` should be a valid selector");

    for li in html.select(&selector) {
        let data_episode_no =  li
            .attr("data-episode-no")
            .assumption("`li` element on `webtoons.com` episode page should always have a `data-episode-no` attribute")?
            .parse::<u16>()
            .assumption("`data-episode-no` attribute on `webtoons.com` episode page should be parseable as a `u16`")?;

        // We look through all the episodes until we find the current one.
        if data_episode_no != episode {
            continue;
        }

        let selector = Selector::parse("a>span.thmb>img._thumbnailImages")
            .expect("`a>span.thmb>img._thumbnailImages` should be a valid selector");

        let url = li
            .select(&selector)
            .next()
            .assumption("`img._thumbnailImages` element should be present in `webtoons.com` episode page `li`")?
            .attr("data-url")
            .assumption("`img._thumbnailImages` element on `webtoons.com` episode page should always have a `data-url` attribute")?;

        let mut thumbnail = Url::parse(url).with_assumption(|| {
            format!("url found on `webtoons.com` episode page should be parseable as a valid url: `{url}`")
        })?;

        thumbnail
            // WHY: This host doesn't need a `referer` header to see the image.
            .set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid url host");

        return Ok(thumbnail);
    }

    assumption!(
        "`webtoons.com` episode page should have a thumbnail for the currently viewed episode"
    );
}

#[inline]
fn is_audio_reader(html: &Html) -> bool {
    let selector = Selector::parse("button#soundControl")
        .expect("`button#soundControl` should be a valid selector");

    // If `<button ... id="soundControl"` exists, then it is an audio reader
    html.select(&selector).next().is_some()
}

#[inline]
fn only_viewable_on_app(html: &Html) -> bool {
    let selector = Selector::parse("div.publishing_wrap>img.qrcode")
        .expect("`div.publishing_wrap>img.qrcode` should be a valid selector");

    // If QR exists, then episode can only be viewed on the app.
    html.select(&selector).next().is_some()
}

fn height(img: ElementRef<'_>) -> Result<u32, Assumption> {
    let value = img
        .value()
        .attr("height")
        .assumption("`img._images` element on `webtoons.com` episode page should always have a `height` attribute")?;

    let height = match value {
        float if let Some((int, fract)) = float.split_once('.') => {
            assume!(
                !fract.is_empty(),
                "fractional component of float height should not be empty, got: `{float}`"
            );
            assume!(
                fract.chars().all(|ch| ch.is_ascii_digit()),
                "fractional component of float height should only contain digits, got: `{fract}`"
            );

            // TODO: Fractional pixel values are truncated. This could cause slight overlap
            // when compositing panels into a single image, but there's no clean solution
            // until we know how common large fractional values are (e.g. `1365.3333...`).
            int.parse::<u32>().with_assumption(|| {
                    format!("integer part of float height on `webtoons.com` episode page should be parseable as a `u32`, got: `{int}`")
            })?
        }
        // Height can also be a whole number: `1280`.
        height => height.parse::<u32>().with_assumption(|| {
            format!("whole-number height on `webtoons.com` episode page should be parseable as a `u32`, got: `{height}`")
        })?,
    };

    // assume!(
    //     // NOTE: from `webtoons.com` episode upload page: `maximum dimensions, 800x1280px`.
    //     // TODO: found canvas `903679` episode 1 which has 1365.3333333333333, so unsure how we want to handle this, as it breaks the stated limits.
    //     height <= 1280,
    //     "`webtoons.com` enforces strict limits of `1280` pixels in height"
    // );

    Ok(height)
}

// NOTE: See `height` and `length` for more info on the possible range of values.
fn width(img: ElementRef<'_>) -> Result<u32, Assumption> {
    let value = img
        .value()
        .attr("width")
        .assumption("`img._images` element on `webtoons.com` episode page should always have a `width` attribute")?;

    let width = match value {
        float if let Some((int, fract)) = float.split_once('.') => {
            assume!(
                !fract.is_empty(),
                "fractional component of float width should not be empty, got: `{float}`"
            );
            assume!(
                fract.chars().all(|ch| ch.is_ascii_digit()),
                "fractional component of float width should only contain digits, got: `{fract}`"
            );

            int.parse::<u32>().with_assumption(|| {
                format!("integer part of float width on `webtoons.com` episode page should be parseable as a `u32`, got: `{int}`")
            })?
        }
        // Width can also be a whole number: `800`.
        width => width.parse::<u32>().with_assumption(|| {
            format!("whole-number width on `webtoons.com` episode page should be parseable as a `u32`, got: `{width}`")
        })?,
    };

    assume!(
        // NOTE: from `webtoons.com` episode upload page: `maximum dimensions, 800x1280px`.
        // TODO: There is a stated limit, but with height as an example, this, too, could be violated on the site.
        width <= 800,
        "`webtoons.com` episode panel width should be at most 800px, got: {width}"
    );

    Ok(width)
}

/// The publish date or datetime of an [`Episode`].
///
/// The precision depends on how the episode was obtained:
///
/// - Via [`Webtoon::episodes()`] without a creator session: date only (`year`, `month`, `day`).
/// - Via [`Webtoon::episodes()`] with a creator session, or [`Webtoon::rss()`]: full datetime.
///
/// Time components ([`hour`](Published::hour), [`minute`](Published::minute),
/// [`second`](Published::second), [`timestamp`](Published::timestamp)) return `None` for
/// date-only values.
#[derive(Debug, Clone, Copy)]
pub struct Published(DateOrDateTime);

impl From<NaiveDate> for Published {
    #[inline]
    fn from(date: NaiveDate) -> Self {
        Self(date.into())
    }
}

impl From<DateTime<Utc>> for Published {
    #[inline]
    fn from(datetime: DateTime<Utc>) -> Self {
        Self(datetime.into())
    }
}

impl Published {
    /// Returns the day of the month. Always available regardless of precision.
    #[inline]
    #[must_use]
    pub fn day(&self) -> u32 {
        let published = &self.0;
        published.day()
    }

    /// Returns the month. Always available regardless of precision.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u32 {
        let published = &self.0;
        published.month()
    }

    /// Returns the year. Always available regardless of precision.
    #[inline]
    #[must_use]
    pub fn year(&self) -> i32 {
        let published = &self.0;
        published.year()
    }

    /// Returns the hour in 24-hour format, or `None` for date-only values.
    #[inline]
    #[must_use]
    pub fn hour(&self) -> Option<u32> {
        let published = &self.0;
        published.hour()
    }

    /// Returns the minute, or `None` for date-only values.
    #[inline]
    #[must_use]
    pub fn minute(&self) -> Option<u32> {
        let published = &self.0;
        published.minute()
    }

    /// Returns the second, or `None` for date-only values.
    #[inline]
    #[must_use]
    pub fn second(&self) -> Option<u32> {
        let published = &self.0;
        published.second()
    }

    /// Returns the Unix timestamp in non-leap seconds since `1970-01-01 00:00:00 UTC`, or `None` for date-only values.
    #[inline]
    #[must_use]
    pub fn timestamp(&self) -> Option<i64> {
        let published = &self.0;
        published.timestamp()
    }
}

/// A single panel of an [`Episode`], obtained via [`Episode::panels()`].
#[allow(unused)] // Not all fields are used with the base feature set.
#[derive(Debug, Clone)]
pub struct Panel {
    url: Url,
    episode: u16,
    number: u16,
    ext: String,
    bytes: Vec<u8>,
    height: u32,
    width: u32,
}

impl Panel {
    /// Returns the URL of this [`Panel`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(843910, Type::Canvas).await?.expect("`843910` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for panel in episode.panels().await? {
    ///         println!("url: {}", panel.url());
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn url(&self) -> &str {
        let panel = self;
        panel.url.as_str()
    }

    #[cfg(feature = "download")]
    async fn download(
        &mut self,
        client: &crate::platform::webtoons::Client,
    ) -> Result<(), reqwest::Error> {
        let panel = self;
        panel.bytes = client.download_panel(&panel.url).await?;
        Ok(())
    }
}

fn panels(html: &Html, episode: u16) -> Result<Vec<Panel>, Assumption> {
    if is_audio_reader(html) {
        return Ok(Vec::new());
    }

    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut panels = Vec::new();

    for (number, img) in html.select(&selector).enumerate() {
        let data_url = img
            .value()
            .attr("data-url")
            // Some urls contain html encoded entities (e.g. https://webtoon-phinf.pstatic.net/20221005_56/1664941257466o9LqU_JPEG/Mom-I&#39;m-Sorry_DE_EP002_01_02.jpg).
            // If we do not clean these, then the parsing of the `ext` will fail
            // as the part after the `#` will be part of fragment, not the path.
            .map(html_escape::decode_html_entities)
            .assumption("`img._images` element on `webtoons.com` episode page should always have a `data-url` attribute")?;

        let mut url = Url::parse(&data_url).with_assumption(|| {
            format!("url found on `webtoons.com` episode page should be parseable as a valid url: `{data_url}`")
        })?;

        // WHY: This host doesn't need a `referer` header to see the image.
        url.set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid url host");

        let ext =  url
            .path()
            .split('.')
            .next_back()
            .map(|ext| ext.to_owned())
            .with_assumption(|| format!("`webtoons.com` episode page panel image url should have a file extension, got: `{url}`"))?;

        // NOTE:
        // `gif` is a supported format in some instances, despite `webtoons.com`
        // wording that states only JPEG and PNG are accepted.
        assume_matches!(
            ext.as_str(),
            "jpeg" | "JPEG" | "png" | "PNG" | "jpg" | "JPG" | "gif" | "GIF",
            "`webtoons.com` episode panel image should be jpeg, png, or gif"
        );

        panels.push(Panel {
            url,
            episode,
            // Panels are 1-indexed.
            number: u16::try_from(number + 1)
                .assumption("`webtoons.com` episode should not have more than 65,535 panels")?,
            height: height(img)?,
            width: width(img)?,
            ext,
            bytes: Vec::new(),
        });
    }

    assume!(
        !panels.is_empty(),
        "`webtoons.com` episode should have at least one panel"
    );

    Ok(panels)
}

/// The downloaded panels of an [`Episode`], obtained via [`Episode::download()`].
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{error::Error, Client, Type};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let webtoon = client.webtoon(961, Type::Original).await?.expect("`961` exists");
///
/// if let Some(episode) = webtoon.episode(1).await? {
///     let panels = episode.download().await?;
///     assert_eq!(52 , panels.count());
///     # return Ok(());
/// }
/// # unreachable!("should have entered the episode block and returned");
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "download")]
#[derive(Debug, Clone)]
pub struct DownloadedPanels {
    panels: Vec<Panel>,
    height: u32,
    width: u32,
}

#[cfg(feature = "download")]
impl DownloadedPanels {
    /// Returns the number of panels in this [`Episode`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(961, Type::Original).await?.expect("`961` exists");
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let panels = episode.download().await?;
    ///     assert_eq!(99 , panels.count());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        let panels = self;
        panels.panels.len()
    }
}

#[cfg(feature = "download")]
use crate::platform::webtoons::error::SavePanelError;
#[cfg(feature = "download")]
use image::{GenericImageView, ImageFormat, RgbaImage};
#[cfg(feature = "download")]
use tokio::io::AsyncWriteExt;

#[cfg(feature = "download")]
impl DownloadedPanels {
    /// Saves all panels as a single vertically composited PNG image.
    ///
    /// Always saves as PNG regardless of the original panel format. Creates `path` and any
    /// missing parent directories if they do not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(2960, Type::Original).await?.expect("`2960` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let panels = episode.download().await?;
    ///     panels.save_single("panels/").await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    #[expect(
        clippy::missing_panics_doc,
        reason = "should only panic if internal logic is wrong, and should be checked in CI"
    )]
    pub async fn save_single<P>(&self, path: P) -> Result<(), SavePanelError>
    where
        P: AsRef<std::path::Path> + Send,
    {
        let path = path.as_ref();
        tokio::fs::create_dir_all(path).await?;

        let panels = self;

        let first = panels
            .panels
            .first()
            .expect("`DownloadedPanels::images` should never be empty");

        let episode = first.episode;
        let width = panels.width;
        let height = panels.height;

        let path = path.join(episode.to_string()).with_extension("png");

        tokio::fs::File::create(&path).await?;

        let mut single = RgbaImage::new(width, height);

        let mut offset = 0;

        for panel in &panels.panels {
            let bytes = panel.bytes.as_slice();

            let image = image::load_from_memory(bytes).assumption(
                "`webtoons.com` episode panel image should be loadable by the `image` crate",
            )?;

            for (x, y, pixels) in image.pixels() {
                single.put_pixel(x, y + offset, pixels);
            }

            offset += image.height();
        }

        match tokio::task::spawn_blocking(move || single.save_with_format(path, ImageFormat::Png))
            .await
            .expect("tokio task saving episode image to disk should not panic")
        {
            Ok(()) => Ok(()),
            Err(image::ImageError::IoError(err)) => Err(SavePanelError::IoError(err)),
            Err(err) => unreachable!(
                "saving episode image to disk should only produce `IoError`, got: `{err}`"
            ),
        }
    }

    /// Saves each panel as an individual file under `path`, named `{episode}-{panel}`.
    ///
    /// For example, panel 1 of episode 34 is saved as `34-1.jpeg`. The file extension matches
    /// the original panel format. Creates `path` and any missing parent directories if they
    /// do not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(2960, Type::Original).await?.expect("`2960` exists");
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let panels = episode.download().await?;
    ///     panels.save_multiple("panels/").await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_multiple<P>(&self, path: P) -> Result<(), SavePanelError>
    where
        P: AsRef<std::path::Path> + Send,
    {
        let path = path.as_ref();
        tokio::fs::create_dir_all(path).await?;

        let panels = self;

        for panel in &panels.panels {
            let name = format!("{}-{}", panel.episode, panel.number);
            let path = path.join(name).with_extension(&panel.ext);

            let mut file = tokio::fs::File::create(&path).await?;

            let bytes = panel.bytes.as_slice();

            file.write_all(bytes).await?;
        }

        Ok(())
    }
}
