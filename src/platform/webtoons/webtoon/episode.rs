//! Module containing things related to an episode on `webtoons.com`.

use anyhow::{Context, anyhow};
use chrono::{DateTime, Utc};
use image::{GenericImageView, ImageFormat, RgbaImage};
use parking_lot::RwLock;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use super::post::{PinRepresentaion, Posts};

use super::Webtoon;
use crate::platform::webtoons::dashboard::episodes::DashboardStatus;
use crate::platform::webtoons::webtoon::post::Post;
use crate::platform::webtoons::webtoon::post::id::Id;
use crate::platform::webtoons::{
    client::Client,
    errors::{ClientError, DownloadError, EpisodeError, PostError},
    meta::Scope,
};

/// Represents a collection of episodes.
///
/// This type is not constructed directly, but via [`Webtoon::episodes()`].
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
#[derive(Debug)]
pub struct Episodes {
    pub(crate) count: u16,
    pub(crate) episodes: Vec<Episode>,
}

impl Episodes {
    /// Returns the count of the episodes retrieved.
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn count(&self) -> u16 {
        self.count
    }

    /// Gets the episode from passed in value if it exists.
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(4470, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// assert_eq!(Some(1), episodes.episode(1).map(|episode| episode.number()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn episode(&self, episode: u16) -> Option<&Episode> {
        // PERF: If in the process of making the Vec we can insert into the index that the number is, then we can use
        // `get(episode)` instead. As of now, the episodes can be in any order, so we have to search through and find
        // the wanted one

        self.episodes
            .iter()
            .find(|__episode| __episode.number == episode)
    }
}

impl From<Vec<Episode>> for Episodes {
    fn from(value: Vec<Episode>) -> Self {
        Self {
            count: u16::try_from(value.len()).expect("max episode number should fit within `u16`"),
            episodes: value,
        }
    }
}

impl IntoIterator for Episodes {
    type Item = Episode;

    type IntoIter = <Vec<Episode> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.episodes.into_iter()
    }
}

/// Represents an episode on `webtoons.com`.
///
/// This type is not constructed directly, but gotten via [`Webtoon::episodes()`] or [`Webtoon::episode()`]
///
/// # Validity
///
/// An instance of an `Episode` should always be considered to exist and be a valid episode on the platform.
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(2960, Type::Original).await? else {
///     unreachable!("webtoon is known to exist");
/// };
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
    pub(crate) season: Arc<RwLock<Option<u8>>>,
    pub(crate) title: Arc<RwLock<Option<String>>>,
    pub(crate) published: Option<DateTime<Utc>>,
    // NOTE: The wrapper Option is to indicate if it has been scraped before, and if so, the inner Option is the actual
    // value.
    pub(crate) length: Arc<RwLock<Option<Option<u32>>>>,
    pub(crate) views: Option<u32>,
    pub(crate) thumbnail: Arc<RwLock<Option<Url>>>,
    pub(crate) note: Arc<RwLock<Option<String>>>,
    pub(crate) ad_status: Option<AdStatus>,
    pub(crate) published_status: Option<PublishedStatus>,
    pub(crate) panels: Arc<RwLock<Option<Vec<Panel>>>>,
}

#[expect(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Episode")
            // omitting `webtoon`
            .field("number", &self.number)
            .field("season", &self.season)
            .field("title", &self.title)
            .field("published", &self.published)
            .field("length", &self.length)
            .field("views", &self.views)
            .field("thumbnail", &self.thumbnail)
            .field("note", &self.note)
            .field("ad_status", &self.ad_status)
            .field("published_status", &self.published_status)
            .field("panels", &self.panels)
            .finish()
    }
}

impl Episode {
    /// Returns the episode number.
    ///
    /// This matches up with the `episode_no=` URL query: [`episode_no=25`]
    ///
    /// Distinctly, this could be different from expectations just basing off of the shown episode numbers, as there could
    /// have been episodes deleted that would shift the numbers; this does not necessarily match up with the `#NUMBER` on the episode list.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(7370, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub fn number(&self) -> u16 {
        self.number
    }

    /// Returns the title of the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6532, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
        if let Some(title) = &*self.title.read() {
            Ok(title.clone())
        } else {
            self.scrape().await?;

            let title = self
                .title
                .read()
                .as_deref()
                .map(|title| title.to_string())
                .context("title should have been scraped with the page scrape")?;

            Ok(title)
        }
    }

    /// Returns the season number if a pattern is detected in the episode's title.
    ///
    /// The method attempts to extract the season number by searching for specific patterns within the episode's title.
    ///
    /// The supported patterns are:
    /// - `[Season \d+]`
    /// - `(Season \d+)`
    /// - `[S\d+]`
    /// - `(S\d+)`
    ///
    /// If no season pattern is found, the method will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(95, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
        let title = self.title().await?;
        let season = self::season(&title);
        *self.season.write() = season;
        Ok(season)
    }

    /// Returns the creator note for episode.
    ///
    /// If there is no note found, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(261984, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     if let Some(note) = episode.note().await? {
    ///         assert!(note.starts_with("Find me as Jayessart"));
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn note(&self) -> Result<Option<String>, EpisodeError> {
        if let Some(note) = &*self.note.read() {
            Ok(Some(note.to_string()))
        } else {
            self.scrape().await?;

            let note = self
                .note
                .read()
                .as_deref()
                .map(|note| note.to_string())
                .context("note should have been scraped with the page scrape")?;

            Ok(Some(note))
        }
    }

    /// Returns the sum of the vertical length in pixels.
    ///
    /// If the page cannot be viewed publicly, for example its behind fast-pass, it will return `None`. It can also be
    /// `None` for some episodes that have audio or GIFs.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
        // TODO: This can be None even when it has scraped before. Might need to make Option<Option<>>?
        if let Some(length) = *self.length.read() {
            Ok(length)
        } else {
            self.scrape().await?;

            let length = self
                .length
                .read()
                .context("length should have been scraped with the page scrape")?;

            Ok(length)
        }
    }

    /// Returns the published timestamp of the episode.
    ///
    /// It returns as `Some` if the episode is publicly available or has a set publish date. Otherwise, it returns `None` if the episode is unpublished.
    ///
    /// # Behavior
    ///
    /// - **Original vs Canvas Episodes**:
    ///   - **Original Webtoons**: For episodes from an Original series, this method will always return `Some` for free episodes, since Originals follow a standard publishing schedule.
    ///   - **Canvas Webtoons (No Session)**: For Canvas episodes, if no session is provided to the [`Client`], it will also return `Some` for the publicly available episodes.
    ///   - **Canvas Webtoons (With Session)**: If a valid creator session is provided for a Canvas webtoon, it may return `None` if the episode is unpublished (i.e., still in draft form).
    ///
    /// - **Important Caveat**:
    ///   - This method **only returns a value** when the episode is accessed via the [`Webtoon::episodes()`] method, which retrieves all episodes, including unpublished ones when available. If the episode is retrieved using [`Webtoon::episode()`], this method will always return `None`, even if the episode is published.
    ///   - Using [`Webtoon::episodes()`] ensures that published episodes return accurate timestamps. For episodes retrieved without a valid creator session, the published time will be available but may default to **2:00 AM UTC** on the publication date due to webtoon page limitations.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let mut episodes = webtoon.episodes().await?.into_iter();
    ///
    /// if let Some(episode) = episodes.next() {
    ///     assert_eq!(Some(1745892000000), episode.published());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn published(&self) -> Option<i64> {
        self.published.map(|datetime| datetime.timestamp_millis())
    }

    /// Returns the view count for the episode.
    ///
    /// It will return as `Some` if available, or `None` if the view count is not accessible.
    ///
    /// # Behavior
    ///
    /// - **Original vs. Canvas Episodes**:
    ///   - **Original Webtoons**: For episodes from an Original series will always return `None`.
    ///   - **Canvas Webtoons (No Session)**: Will always return `None`.
    ///   - **Canvas Webtoons (With Session)**: If a valid creator session is provided, the method will include views for all episodes, including those behind fast-pass, ad walls, or even unpublished episodes, provided the episode is retrieved using [`Webtoon::episodes()`].
    ///
    /// - **Important Caveat**:
    ///   - **Views will always return `None`** when using the [`Webtoon::episode()`] method to retrieve a single episode. To get the view count, you **must use** [`Webtoon::episodes()`], which fetches all episodes in bulk and provides view count data when available.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("my-session");
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let mut episodes = webtoon.episodes().await?.into_iter();
    ///
    /// if let Some(episode) = episodes.next() {
    ///     println!("episode {} has {:?} views", episode.number(), episode.views());
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn views(&self) -> Option<u32> {
        self.views
    }

    /// Returns the like count for the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("episode {} has {} likes", episode.number(), episode.likes().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn likes(&self) -> Result<u32, EpisodeError> {
        let response = self.webtoon.client.get_likes_for_episode(self).await?;

        let contents = response.result.contents.first().context(
            "`contents` field in likes api didn't have a 0th element and it should always have one",
        )?;

        let likes = contents
            .reactions
            .first()
            .map(|likes| likes.count)
            .unwrap_or_default();

        Ok(likes)
    }

    /// Returns the comment and reply count for the episode.
    ///
    /// Tuple is returned as `(comments, replies)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub async fn comments_and_replies(&self) -> Result<(u32, u32), PostError> {
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 1, PinRepresentaion::None)
            .await?;

        let comments = response.result.active_root_post_count;
        let replies = response.result.active_post_count - comments;

        Ok((comments, replies))
    }

    /// Retrieves the direct (top-level) comments for the episode, sorted from newest to oldest.
    ///
    /// There are no duplicate comments, and only direct replies (top-level) are fetched, not the nested replies.
    ///
    /// Direct replies that have been deleted (but have replies) will still be included with a message indicating the deletion. Comments deleted without replies will not be included.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for post in episode.posts().await? {
    ///         println!("{} left a comment on episode {} of {}", post.poster().username(), episode.number(), webtoon.title().await?);
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts(&self) -> Result<Posts, PostError> {
        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut posts = HashSet::new();

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 100, PinRepresentaion::None)
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first posts.
        for post in response.result.posts {
            posts.insert(Post::try_from((self, post))?);
        }

        // Get any remaining.
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100, PinRepresentaion::None)
                .await?;

            for post in response.result.posts {
                posts.replace(Post::try_from((self, post))?);
            }

            next = response.result.pagination.next;
        }

        // Get is_top/isPinned info.
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 1, PinRepresentaion::Distinct)
            .await?;

        for post in response.result.tops {
            posts.replace(Post::try_from((self, post))?);
        }

        let posts: Vec<Post> = posts.into_iter().collect();
        let mut posts = Posts { posts };

        posts.sort_by_newest();

        Ok(posts)
    }

    /// Iterates over all direct (top-level) comments for the episode and applies a callback function to each post.
    ///
    /// This method is useful in scenarios where memory constraints are an issue, as it avoids loading all posts into memory at once. Instead, each post is processed immediately as it is retrieved, making it more memory-efficient than [`posts()`](Episode::posts()).
    ///
    /// # Limitations
    ///
    /// - **Duplicate Posts**:
    ///   - Due to potential API inconsistencies during pagination, this method cannot guarantee that duplicate posts will be filtered out.
    ///
    /// - **Publish Order**:
    ///   - The order in which posts are published may not be respected, as the posts are fetched and processed in batches that may appear out of order.
    ///
    /// - **`is_top` Info**:
    ///   - This information will only be added at the very end of iteration. Previous posts info might not have correct `is_top` status, due to the nature of how webtoons' API's work.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(1046567, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///    episode.posts_for_each(async |post| {
    ///        println!("{} left a comment on episode {}", post.poster().username(), episode.number());
    ///    }).await?;
    ///    # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts_for_each<C: AsyncFn(Post)>(&self, closure: C) -> Result<(), PostError> {
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 100, PinRepresentaion::None)
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first posts
        for post in response.result.posts {
            closure(Post::try_from((self, post))?).await;
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100, PinRepresentaion::None)
                .await?;

            for post in response.result.posts {
                closure(Post::try_from((self, post))?).await;
            }

            next = response.result.pagination.next;
        }

        // Gets `is_top/isPinned` info.
        //
        // NOTE: This is after the regular posts as more often than not, if using
        // a collection, user will use a HashSet::replace, which would update
        // the previously gotten posts with the pinned info.
        //
        // If user is directly inserting into database, the data should also be
        // updated accordingly.
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 1, PinRepresentaion::Distinct)
            .await?;

        for post in response.result.tops {
            closure(Post::try_from((self, post))?).await;
        }

        Ok(())
    }

    /// Retrieves the direct (top-level) comments for the episode until the specified post `id` is encountered.
    ///
    /// This method can be used for fetching the most recent posts for an episode, with the assumption that the post
    /// with the given `id` was not deleted, since posts are fetched from newest to oldest.
    ///
    /// If the post with the `id` was deleted or missing, this method may perform a full scan of the episode. For cases
    /// where the post's date is known but the `id` is uncertain to exist, use [`Episode::posts_till_date()`].
    ///
    /// # Limitations
    ///
    /// - **Deleted Posts**: Posts without replies that have been deleted are not returned in the results, which may lead to a situation where the post with the given `id` is never found, causing the method to scan the entire episode.
    /// - **Performance Consideration**: If the post is near the end of the episode's comments, the method may need to scan all the way through to find the `id`, which can impact performance for episodes with large comment sections.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for post in episode.posts_till_id("GW-epicom:0-c_843910_1-g").await? {
    ///         println!("Comment by {}: {}", post.poster().username(), post.body().contents());
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts_till_id(&self, id: &str) -> Result<Posts, PostError> {
        let id = Id::from_str(id).map_err(|err| PostError::Unexpected(err.into()))?;

        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has a `Client` that has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut posts = HashSet::new();

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 100, PinRepresentaion::None)
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first posts
        for post in response.result.posts {
            if post.id == id {
                return Ok(Posts {
                    posts: posts.into_iter().collect(),
                });
            }

            posts.insert(Post::try_from((self, post))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100, PinRepresentaion::None)
                .await?;

            for post in response.result.posts {
                if post.id == id {
                    return Ok(Posts {
                        posts: posts.into_iter().collect(),
                    });
                }

                posts.insert(Post::try_from((self, post))?);
            }

            next = response.result.pagination.next;
        }

        let mut posts = Posts {
            posts: posts.into_iter().collect(),
        };

        posts.sort_by_newest();

        Ok(posts)
    }

    /// Retrieves the direct (top-level) comments for the episode until posts older than the specified `date` are encountered.
    ///
    /// This method can be used as an optimization to get only the most recent posts from a given date onward.
    /// It ensures all posts from the given date are retrieved, even if multiple posts share the same timestamp.
    ///
    /// # Limitations
    ///
    /// - **Duplicate Timestamps**: Posts can have the same creation date, so the method ensures that all posts with a given timestamp are returned before stopping.
    /// - **Performance**: Similar to [`Episode::posts_till_id()`], this method may impact performance for episodes with many comments, especially if the `date` is far in the past.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     // UNIX timestamp
    ///     for post in episode.posts_till_date(1729582054).await? {
    ///         println!("Comment by {}: {}", post.poster().username(), post.body().contents());
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts_till_date(&self, date: i64) -> Result<Posts, PostError> {
        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut posts = HashSet::new();

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 100, PinRepresentaion::None)
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first posts
        for post in response.result.posts {
            if post.created_at < date {
                return Ok(Posts {
                    posts: posts.into_iter().collect(),
                });
            }

            posts.insert(Post::try_from((self, post))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100, PinRepresentaion::None)
                .await?;

            for post in response.result.posts {
                if post.created_at < date {
                    return Ok(Posts {
                        posts: posts.into_iter().collect(),
                    });
                }

                posts.insert(Post::try_from((self, post))?);
            }

            next = response.result.pagination.next;
        }

        let mut posts = Posts {
            posts: posts.into_iter().collect(),
        };

        posts.sort_by_newest();

        Ok(posts)
    }

    /// Returns a list of panels for the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
        if let Some(panels) = &*self.panels.read() {
            Ok(panels.to_owned())
        } else {
            self.scrape().await?;

            let panels = self
                .panels
                .read()
                .as_ref()
                .context("panels should have been scraped with the page scrape")?
                .to_owned();

            Ok(panels)
        }
    }

    /// Returns the thumbnail URL for episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
        if let Some(thumbnail) = &*self.thumbnail.read() {
            Ok(thumbnail.to_string())
        } else {
            self.scrape().await?;

            let thumbnail = self
                .thumbnail
                .read()
                .as_ref()
                .context("thumbnail should have been scraped with the page scrape")?
                .to_string();

            Ok(thumbnail)
        }
    }

    /// Returns the [`PublishedStatus`] for the episode, indicating whether the episode is published, a draft, or removed.
    ///
    /// This can only be accurately determined when using [`Webtoon::episodes()`]. If [`Webtoon::episode()`] is used,
    /// this will always return `None`, as the necessary metadata to determine an episode's published status is not available
    /// through [`Webtoon::episode()`].
    ///
    /// The possible states are:
    /// - [`Published`](variant@PublishedStatus::Published) - The episode is publicly available in some capacity (ad, fast-pass, or fully public).
    /// - [`Draft`](variant@PublishedStatus::Draft) - The episode is not published in any form yet (it is in draft status).
    /// - [`Removed`](variant@PublishedStatus::Removed) - The episode has been removed from publication.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, webtoon::episode::PublishedStatus, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6889, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// if let Some(episode) = episodes.episode(1) {
    ///     match episode.published_status() {
    ///             Some(PublishedStatus::Published) => println!("Episode is published."),
    ///             Some(PublishedStatus::Draft) => println!("Episode is still a draft."),
    ///             Some(PublishedStatus::Removed) => println!("Episode has been removed."),
    ///             None => unreachable!("must use `Webtoon::episodes()`!"),
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn published_status(&self) -> Option<PublishedStatus> {
        self.published_status
    }

    /// Returns the episode's current ad status.
    ///
    /// This information is only available if a session is provided, and the Webtoon in question was created by the user of the session.
    /// In any other scenario, this method returns `None`. To retrieve this data, the `episodes` function must be used when getting episode data.
    ///
    /// The possible states are:
    /// - [`Yes`](variant@AdStatus::Yes) - The episode is currently behind an ad.
    /// - [`No`](variant@AdStatus::No) - The episode is no longer behind an ad, but once was.
    /// - [`Never`](variant@AdStatus::Never) - The episode was never behind any kind of ad.
    ///
    /// # Original Series:
    /// For original webtoons, it's not possible to determine the ad status from the public episode listing alone.
    /// Generally, any random original episode may have been behind fast-pass, but initial release episodes (which are typically not behind fast-pass) would be indistinguishable.
    /// Therefore, for Original series, this method will always return `None`.
    ///
    /// # Canvas Series:
    /// For canvas webtoons created by the session's user, the ad status can be retrieved and returned if applicable.
    /// If no session is provided and only public info is used, this will always return `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, webtoon::episode::AdStatus, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// if let Some(episode) = episodes.episode(1) {
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
    pub fn ad_status(&self) -> Option<AdStatus> {
        self.ad_status
    }

    /// Likes the episode on behalf of the user associated with the current session.
    ///
    /// Allows the user (via their session) to like a specific episode. If no session is present, or is invalid, will return an [`EpisodeError`].
    ///
    /// If episode is already liked, it will do nothing.
    ///
    /// # Behavior:
    /// - **Session Required**: The method will attempt to like the episode on behalf of the user tied to the current session.
    /// - **Webtoon Ownership**: If the episode belongs to the current user’s own Webtoon, it will still process the request without issue.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::{Error, EpisodeError, ClientError}, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     match episode.like().await {
    ///         Ok(_) => println!("Liked episode {} of {}!", episode.number(), webtoon.title().await?),
    ///         Err(EpisodeError::ClientError(ClientError::NoSessionProvided)) => println!("Provide a session!"),
    ///         Err(EpisodeError::ClientError(ClientError::InvalidSession)) => println!("Session given was invalid!"),
    ///         Err(err) => println!("Error: {err}"),
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn like(&self) -> Result<(), EpisodeError> {
        self.webtoon.client.like_episode(self).await?;
        Ok(())
    }

    /// Unlikes the episode on behalf of the user associated with the current session.
    ///
    /// Allows the user (via their session) to unlike a specific episode. If no session is present, or is invalid, will return an [`EpisodeError`].
    ///
    /// If episode is already not liked, it will do nothing.
    ///
    /// # Behavior:
    /// - **Session Required**: The method will attempt to unlike the episode on behalf of the user tied to the current session.
    /// - **Webtoon Ownership**: If the episode belongs to the current user’s own Webtoon, it will still process the request without issue.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::{Error, EpisodeError, ClientError}, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     match episode.unlike().await {
    ///         Ok(_) => println!("Uniked episode {} of {}!", episode.number(), webtoon.title().await?),
    ///         Err(EpisodeError::ClientError(ClientError::NoSessionProvided)) => println!("Provide a session!"),
    ///         Err(EpisodeError::ClientError(ClientError::InvalidSession)) => println!("Session given was invalid!"),
    ///         Err(err) => println!("Error: {err}"),
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unlike(&self) -> Result<(), EpisodeError> {
        self.webtoon.client.unlike_episode(self).await?;
        Ok(())
    }

    /// Posts a top-level comment on the episode.
    ///
    /// This method allows users to leave a comment on an episode. The comment can be marked as a spoiler.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::{Error, PostError, ClientError}, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     match episode.post("Loved this episode!", false).await {
    ///         Ok(_) => println!("Left comment on episode {} of {}!", episode.number(), webtoon.title().await?),
    ///         Err(PostError::ClientError(ClientError::NoSessionProvided)) => println!("Provide a session!"),
    ///         Err(PostError::ClientError(ClientError::InvalidSession)) => println!("Session given was invalid!"),
    ///         Err(err) => println!("Error: {err}"),
    ///     }
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post(&self, body: &str, is_spoiler: bool) -> Result<(), PostError> {
        let page_id = format!(
            "{}_{}_{}",
            match self.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            self.webtoon.id,
            self.number
        );

        let spoiler_filter = if is_spoiler { "ON" } else { "OFF" };

        let body = json!(
            {
                "pageId": page_id,
                "settings":{
                    "reply": "ON",
                    "reaction": "ON",
                    "spoilerFilter": spoiler_filter
                },
                "body": body
            }
        );

        let token = self.webtoon.client.get_api_token().await?;

        let session = self
            .webtoon
            .client
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .unwrap_or_default();

        self.webtoon
            .client
            .http
            .post("https://www.webtoons.com/p/api/community/v2/post")
            .json(&body)
            .header("Service-Ticket-Id", "epicom")
            .header("Api-Token", token)
            .header("Cookie", format!("NEO_SES={session}"))
            .send()
            .await?;

        Ok(())
    }

    /// Will download the panels of episode.
    ///
    /// This returns a [`Panels`], which offers ways to save to disk.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6889, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub async fn download(&self) -> Result<Panels, EpisodeError> {
        use tokio::sync::Semaphore;

        let mut panels = if let Some(panels) = &*self.panels.read() {
            panels.to_owned()
        } else {
            self.scrape().await?;

            self.panels
                .read()
                .as_ref()
                .context("panels should have been scraped with the page scrape")?
                .to_owned()
        };

        // PERF: Download N panels at a time. Without this it will be a sequential.
        let semaphore = Semaphore::new(100);

        let mut height = 0;
        let mut width = 0;

        for panel in &mut panels {
            let semaphore = semaphore
                .acquire()
                .await
                .context("failed to acquire sepmahore when downloading panels")?;

            panel.download(&self.webtoon.client).await?;

            drop(semaphore);

            height += panel.height;
            width = width.max(panel.width);
        }

        Ok(Panels {
            images: panels,
            height,
            width,
        })
    }
}

// Internal use only
impl Episode {
    pub(crate) fn new(webtoon: &Webtoon, number: u16) -> Self {
        Self {
            webtoon: webtoon.clone(),
            number,
            season: Arc::new(RwLock::new(None)),
            title: Arc::new(RwLock::new(None)),
            // NOTE: Currently there is no way to get this info from an episodes page.
            // The only sources are the dashboard episode list data, and the episode list from the webtoons page.
            // This could be gotten, in theory, with the webtoons page episode data, but caching the episodes
            // would lead to a large refactor and be slow for when only getting one episodes data.
            // For now will just return None until a solution can be landed on.
            published: None,
            length: Arc::new(RwLock::new(None)),
            thumbnail: Arc::new(RwLock::new(None)),
            note: Arc::new(RwLock::new(None)),
            panels: Arc::new(RwLock::new(None)),
            views: None,
            ad_status: None,
            published_status: None,
        }
    }

    /// Scrapes episode page, getting `note`, `length`, `title`, `thumbnail` and the urls for the panels.
    async fn scrape(&self) -> Result<(), EpisodeError> {
        let html = self
            .webtoon
            .client
            .get_episode(&self.webtoon, self.number)
            .await?;

        let title = title(&html).context("Episode title failed to be parsed")?;
        *self.title.write() = Some(title);

        let thumbnail = thumbnail(&html, self.number) //
            .context("Episode thumbnail failed to be parsed")?;
        *self.thumbnail.write() = Some(thumbnail);

        let length = length(&html) //
            .context("Episode length failed to be parsed")?;
        *self.length.write() = Some(length);

        let note = note(&html) //
            .context("Episode note failed to be parsed")?;
        *self.note.write() = note;

        let panels =
            panels(&html, self.number).context("Episode panel urls failed to be parsed")?;
        *self.panels.write() = Some(panels);

        Ok(())
    }

    /// Returns `true` id episode exists, `false` if not. Returns `PostError` if there was an error.
    pub(super) async fn exists(&self) -> Result<bool, ClientError> {
        self.webtoon.client.check_if_episode_exists(self).await
    }
}

pub(crate) fn season(title: &str) -> Option<u8> {
    // [Season 3]
    let square_brackets_long =
        Regex::new(r"\[Season (?P<season>\d+)\]").expect("regex should be valid");

    if let Some(capture) = square_brackets_long.captures(title.as_ref()) {
        let season = capture["season"]
            .parse::<u8>()
            .expect(r"regex match on `\d+` so should be parsable as an int");

        return Some(season);
    }

    // [S3]
    let square_brackets_short = Regex::new(r"\[S(?P<season>\d+)\]").expect("regex should be valid");

    if let Some(capture) = square_brackets_short.captures(title.as_ref()) {
        let season = capture["season"]
            .parse::<u8>()
            .expect(r"regex match on `\d+` so should be parsable as an int");

        return Some(season);
    }

    // (S3)
    let parens_short = Regex::new(r"\(S(?P<season>\d+)\)").expect("regex should be valid");

    if let Some(capture) = parens_short.captures(title.as_ref()) {
        let season = capture["season"]
            .parse::<u8>()
            .expect(r"regex match on `\d+` so should be parsable as an int");

        return Some(season);
    }

    // (Season 3)
    let parens_long = Regex::new(r"\(Season (?P<season>\d+)\)").expect("regex should be valid");

    if let Some(capture) = parens_long.captures(title.as_ref()) {
        let season = capture["season"]
            .parse::<u8>()
            .expect(r"regex match on `\d+` so should be parsable as an int");

        return Some(season);
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

/// Represents an [`Episode`]'s ad status.
#[derive(Debug, Clone, Copy)]
pub enum AdStatus {
    /// Episode is currently behind an ad.
    Yes,
    /// Episode is not currently behind an ad.
    No,
    /// Episode was never behind an ad.
    Never,
}

/// Represents the publication status of an episode.
///
/// The `PublishedStatus` enum indicates the current state of an episode. It is used to differentiate between episodes
/// that are publicly available, those that are still drafts, and those that have been removed from publication.
///
/// ### Variants:
///
/// - `Published`:
///   The episode is available to the public. This includes episodes behind ad or fast-pass paywalls.
///
/// - `Draft`:
///   The episode is not yet published in any capacity. This means it hasn't been made available to the public or
///   put behind ad/fast-pass options.
///
/// - `Removed`:
///   The episode was previously published but has since been removed. This might happen due to takedowns, content issues, or other reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishedStatus {
    /// The episode is available to the public. This includes episodes behind ad or fast-pass paywalls.
    Published,
    /// The episode is not yet published in any capacity. This means it hasn't been made available to the public or put behind ad/fast-pass options.
    Draft,
    /// The episode was previously published but has since been removed. This might happen due to takedowns, content issues, or other reasons.
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

fn title(html: &Html) -> Result<String, EpisodeError> {
    let selector = Selector::parse("div.subj_info>.subj_episode") //
        .expect("`div.subj_info>.subj_episode` should be a valid selector");

    let title = html
            .select(&selector)
            .next()
            .context("`.subj_episode` is missing: episode page should always contain a title for the episode")?
            .text()
            .next()
            .context("`.subj_episode` was found but no text was present")?;

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn length(html: &Html) -> Result<Option<u32>, EpisodeError> {
    if is_audio_reader(html) {
        return Ok(None);
    }

    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut length = 0;

    for img in html.select(&selector) {
        length += img
            .value()
            .attr("height")
            .context("`height` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`height` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;
    }

    if length == 0 {
        return Err(EpisodeError::NoPanelsFound);
    }

    Ok(Some(length))
}

fn note(html: &Html) -> Result<Option<String>, EpisodeError> {
    let selector = Selector::parse(r".creator_note>.author_text") //
        .expect("`.creator_note>.author_text` should be a valid selector");

    let Some(selection) = html.select(&selector).next() else {
        return Ok(None);
    };

    let note = selection
        .text()
        .next()
        .context("`.author_text` was found but no text was present")?
        .to_owned();

    Ok(Some(note))
}

fn thumbnail(html: &Html, episode: u16) -> Result<Url, EpisodeError> {
    let selector =
        Selector::parse(r"div.episode_lst>div.episode_cont>ul>li") //
            .expect(r"`div.episode_lst>div.episode_cont>ul>li` should be a valid selector");

    for li in html.select(&selector) {
        let data_episode_no = li
            .attr("data-episode-no")
            .context("`data-episode-no` is missing, `li` should always have one")?
            .parse::<u16>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        if data_episode_no != episode {
            continue;
        }

        let img_selection = Selector::parse("a>span.thmb>img._thumbnailImages")
            .expect("`a>span.thmb>img._thumbnailImages` should be a valid selector");

        let mut img = li.select(&img_selection);

        let url = img
            .next()
            .context(
                "`img._thumbnailImages` is missing: episode page page should have at least one",
            )?
            .attr("data-url")
            .context("`data-url` is missing, `img._thumbnailimages` should always have one")?;

        let mut thumbnail = Url::parse(url).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        thumbnail
            // This host doesn't need a `referer` header to see the image.
            .set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

        return Ok(thumbnail);
    }

    Err(EpisodeError::NoThumbnailFound)
}

fn is_audio_reader(html: &Html) -> bool {
    let selector = Selector::parse("button#soundControl")
        .expect("`button#soundControl` should be a valid selector");

    // If `<button ... id="soundControl"` exists, then it is an audio reader
    html.select(&selector).next().is_some()
}

/// Represents a single panel for an episode.
///
/// This type is not constructed directly, but gotten via [`Episode::panels()`](Episode::panels()).
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
    /// Returns the url for the panel.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    async fn download(&mut self, client: &Client) -> Result<(), EpisodeError> {
        let bytes = client
            .http
            .get(self.url.as_str())
            .send()
            .await?
            .bytes()
            .await?;

        self.bytes = bytes.to_vec();

        Ok(())
    }
}

fn panels(html: &Html, episode: u16) -> Result<Vec<Panel>, EpisodeError> {
    if is_audio_reader(html) {
        return Ok(Vec::new());
    }

    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut panels = Vec::new();

    for (number, img) in html.select(&selector).enumerate() {
        let height = img
            .value()
            .attr("height")
            .context("`height` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`height` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let width = img
            .value()
            .attr("width")
            .context("`width` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`width` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let url = img
            .value()
            .attr("data-url")
            .context("`data-url` is missing, `img._images` should always have one")?;

        let mut url = Url::parse(url).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        url.set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

        let ext = url
            .path()
            .split('.')
            .nth(1)
            .with_context(|| format!("`{url}` should end in an extension but didn't"))?
            .to_string();

        panels.push(Panel {
            url,

            episode,
            // Enumerate starts at 0. +1 so that it starts at one.
            number: u16::try_from(number + 1)
                .context("there shouldn't be more than 65,536 panels for an episode")?,
            height,
            width,
            ext,
            bytes: Vec::new(),
        });
    }

    if panels.is_empty() {
        return Err(EpisodeError::Unexpected(anyhow!(
            "Failed to find a single panel on episode page"
        )));
    }

    Ok(panels)
}

/// Represents all the panels for an episode.
///
/// This type is not constructed directly, but gotten via [`Episode::panels()`](crate::platform::webtoons::webtoon::episode::Episode::panels()).
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(961, Type::Original).await? else {
///     unreachable!("webtoon is known to exist");
/// };
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
#[derive(Debug, Clone)]
pub struct Panels {
    images: Vec<Panel>,
    height: u32,
    width: u32,
}

impl Panels {
    /// Returns how many `Panels` are on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(961, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub fn count(&self) -> usize {
        self.images.len()
    }

    /// Saves all the panels of an episode as a single long image file in PNG format.
    ///
    /// # Behavior
    ///
    /// - Combines all panels of the episode vertically into one long image.
    /// - The output image is always saved as a PNG file, even if the original panels are in a different format (e.g., JPEG), due to JPEG's limitations.
    /// - If the directory specified by `path` does not exist, it will be created along with any required parent directories.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(2960, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub async fn save_single<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        let ext = &self.images[0].ext;
        let episode = self.images[0].episode;
        let width = self.width;
        let height = self.height;

        let path = path.join(episode.to_string()).with_extension(ext);

        File::create(&path)
            .await
            .context("failed to create download file")?;

        let mut single = RgbaImage::new(width, height);

        let mut offset = 0;

        for panel in &self.images {
            let bytes = panel.bytes.as_slice();

            let image = image::load_from_memory(bytes) //
                .context("failed to load image from memory")?;

            for (x, y, pixels) in image.pixels() {
                single.put_pixel(x, y + offset, pixels);
            }

            offset += image.height();
        }

        tokio::task::spawn_blocking(move || single.save_with_format(path, ImageFormat::Png))
            .await
            .context("Failed `spawn_blocking`")?
            .context("Failed to save image to disk")?;

        Ok(())
    }

    /// Saves each panel of the episode to disk, naming the resulting files using the format `EPISODE_NUMBER-PANEL_NUMBER`.
    ///
    /// For example, the first panel of the 34th episode would be saved as `34-1`. The file extension will match the panel's original format.
    ///
    /// # Behavior
    ///
    /// - If the specified directory does not exist, it will be created, along with any necessary parent directories.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(2960, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
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
    pub async fn save_multiple<P>(&self, path: P) -> Result<(), DownloadError>
    where
        P: AsRef<Path> + Send,
    {
        let path = path.as_ref();

        tokio::fs::create_dir_all(path).await?;

        for panel in &self.images {
            let name = format!("{}-{}", panel.episode, panel.number);
            let path = path.join(name).with_extension(&panel.ext);

            let mut file = File::create(&path)
                .await
                .context("failed to create download file")?;

            let bytes = panel.bytes.as_slice();

            file.write_all(bytes).await?;
        }

        Ok(())
    }
}
