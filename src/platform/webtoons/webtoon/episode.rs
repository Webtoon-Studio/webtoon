//! Module containing things related to an episode on `webtoons.com`.

mod page;
pub mod posts;

pub use page::panels::Panel;
pub use page::panels::Panels;

use anyhow::Context;
use chrono::{DateTime, Utc};
use core::fmt;
use parking_lot::RwLock;
use posts::Post;
use regex::Regex;
use scraper::Html;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use std::{hash::Hash, str::FromStr};

use self::page::Page;
use self::posts::Posts;
use crate::platform::webtoons::client::likes::Likes;
use crate::platform::webtoons::client::posts::PostsResult;
use crate::platform::webtoons::client::posts::id::Id;
use crate::platform::webtoons::{
    errors::{ClientError, EpisodeError, PostError},
    meta::Scope,
};

use super::{Webtoon, dashboard::episodes::DashboardStatus};

// NOTE: Alternate comment and reply count API
//GET https://www.webtoons.com/p/api/community/v1/pages/activity/count?pageIds=c_843910_1
// {
//     "status": "success",
//     "result": {
//         "countList": [
//             {
//                 "pageId": "c_843910_1",
//                 "totalRootPostCount": 23,
//                 "totalPostCount": 33,
//                 "totalReactionCount": 0,
//                 "activeRootPostCount": 21,
//                 "activePostCount": 29,
//                 "activeReactionCount": 0,
//                 "activeRootPostReactionCount": 0,
//                 "writePostRestrictionCount": 0
//             }
//         ]
//     }
// }

// NOTE: Alternate likes API for episode
// GET https://www.webtoons.com/api/v1/like/search/counts?serviceId=LINEWEBTOON&contentIds=c_843910_1
// {
//     "result": {
//         "contents": [
//             {
//                 "contentsId": "c_843910_1",
//                 "reactions": [
//                     {
//                         "reactionType": "like",
//                         "count": 1,
//                         "isReacted": false
//                     }
//                 ]
//             }
//         ]
//     },
//     "success": true
// }

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
    pub(crate) page: Arc<RwLock<Option<Page>>>,
    pub(crate) views: Option<u32>,
    pub(crate) ad_status: Option<AdStatus>,
    pub(crate) published_status: Option<PublishedStatus>,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Episode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Episode")
            // omitting `webtoon`
            .field("number", &self.number)
            .field("season", &self.season)
            .field("title", &self.title)
            .field("published", &self.published)
            .field("page", &self.page)
            .field("views", &self.views)
            .field("ad_status", &self.ad_status)
            .field("published_status", &self.published_status)
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
    ///     assert_eq!(15, episode.number());
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
    ///     assert_eq!("Myst, Might, Mayhem", episode.title().await?);
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
            let page = Some(self.scrape().await?);
            let title = page
                .as_ref()
                .map(|page| page.title.clone())
                .context("title should have been scraped with the page scrape")?;
            *self.page.write() = page;
            *self.title.write() = Some(title.clone());
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
    ///     assert!(episode.note().await?.starts_with("Find me as Jayessart"));
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn note(&self) -> Result<Option<String>, EpisodeError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.note.clone())
        } else {
            let page = self.scrape().await?;
            let note = page.note.clone();
            *self.page.write() = Some(page);
            Ok(note)
        }
    }

    /// Returns the sum of the vertical length in pixels.
    ///
    /// If the page cannot be viewed publicly, for example its behind fast-pass, it will return `None`.
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
    ///     assert!(Some(0), episode.length().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn length(&self) -> Result<Option<u32>, EpisodeError> {
        if let Some(page) = &*self.page.read() {
            Ok(Some(page.length))
        } else {
            let page = match self.scrape().await {
                Ok(page) => page,
                Err(EpisodeError::NotViewable) => return Ok(None),
                Err(err) => return Err(err),
            };

            let length = page.length;
            *self.page.write() = Some(page);

            Ok(Some(length))
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
    ///   - **Canvas Webtoons (No Session)**: For Canvas episodes, if no session is provided to the [`Client`](super::Client), it will also return `Some` for the publicly available episodes.
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
    ///     assert!(Some(0), episode.published().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn published(&self) -> Option<i64> {
        self.published.map(|datetime| datetime.timestamp_millis())
    }

    // TODO: Do the rest below

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
    ///     println!("episode {} has {:?} views", episode.number(), episode.views().await?);
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
    /// if let Some(episode) = webtoon.episode(1) {
    ///     println!("episode {} has {} likes", episode.number(), episode.likes().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn likes(&self) -> Result<u32, EpisodeError> {
        let response = self
            .webtoon
            .client
            .get_likes_for_episode(self)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<Likes>(&response).context(response)?;

        let api = api.result.contents.first().context(
        "`contents` field  in likes api didn't have a 0th element and it should always have one",
    )?;

        let likes = api
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
    /// if let Some(episode) = webtoon.episode(1) {
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
            .get_posts_for_episode(self, None, 1)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let comments = api.result.active_root_post_count;
        let replies = api.result.active_post_count - comments;

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
    /// if let Some(episode) = webtoon.episode(1) {
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
            .get_posts_for_episode(self, None, 100)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let mut next: Option<Id> = api.result.pagination.next;

        // Add first posts
        for post in api.result.posts {
            posts.insert(Post::try_from((self, post))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100)
                .await?
                .text()
                .await?;

            let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

            for post in api.result.posts {
                posts.replace(Post::try_from((self, post))?);
            }

            next = api.result.pagination.next;
        }

        // Adds `is_top/isPinned` info. The previous API loses this info but is easier to work with so
        // This extra step to the other API is a one off to get only the top comment info attached to
        // the top 3 posts.
        let page_id = format!(
            "{}_{}_{}",
            self.webtoon.scope.as_single_letter(),
            self.webtoon.id,
            self.number
        );

        let url = format!(
            "https://www.webtoons.com/p/api/community/v1/page/{page_id}/posts/search?pinRepresentation=distinct&prevSize=0&nextSize=1"
        );

        let response = self
            .webtoon
            .client
            .http
            .get(url)
            .header("Service-Ticket-Id", "epicom")
            .send()
            .await
            .map_err(|err| ClientError::Unexpected(err.into()))?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        if let Some(tops) = api.result.tops {
            for post in tops {
                posts.replace(Post::try_from((self, post))?);
            }
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
    /// if let Some(episode) = webtoon.episode(1) {
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
        // Adds `is_top/isPinned` info. The previous API loses this info but is easier to work with so
        // This extra step to the other API is a one off to get only the top comment info attached to
        // the top 3 posts.
        let page_id = format!(
            "{}_{}_{}",
            self.webtoon.scope.as_single_letter(),
            self.webtoon.id,
            self.number
        );

        let url = format!(
            "https://www.webtoons.com/p/api/community/v1/page/{page_id}/posts/search?pinRepresentation=distinct&prevSize=0&nextSize=1"
        );

        let response = self
            .webtoon
            .client
            .http
            .get(url)
            .header("Service-Ticket-Id", "epicom")
            .send()
            .await
            .map_err(|err| ClientError::Unexpected(err.into()))?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        if let Some(tops) = api.result.tops {
            for post in tops {
                closure(Post::try_from((self, post))?).await;
            }
        }

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, None, 100)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let mut next: Option<Id> = api.result.pagination.next;

        // Add first posts
        for post in api.result.posts {
            closure(Post::try_from((self, post))?).await;
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, Some(cursor), 100)
                .await?
                .text()
                .await?;

            let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

            for post in api.result.posts {
                closure(Post::try_from((self, post))?).await;
            }

            next = api.result.pagination.next;
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
    /// if let Some(episode) = webtoon.episode(1) {
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
            .get_posts_for_episode(self, None, 100)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let mut next: Option<Id> = api.result.pagination.next;

        // Add first posts
        for post in api.result.posts {
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
                .get_posts_for_episode(self, Some(cursor), 100)
                .await?
                .text()
                .await?;

            let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

            for post in api.result.posts {
                if post.id == id {
                    return Ok(Posts {
                        posts: posts.into_iter().collect(),
                    });
                }

                posts.insert(Post::try_from((self, post))?);
            }

            next = api.result.pagination.next;
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
    /// if let Some(episode) = webtoon.episode(1) {
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
            .get_posts_for_episode(self, None, 100)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let mut next: Option<Id> = api.result.pagination.next;

        // Add first posts
        for post in api.result.posts {
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
                .get_posts_for_episode(self, Some(cursor), 100)
                .await?
                .text()
                .await?;

            let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

            for post in api.result.posts {
                if post.created_at < date {
                    return Ok(Posts {
                        posts: posts.into_iter().collect(),
                    });
                }

                posts.insert(Post::try_from((self, post))?);
            }

            next = api.result.pagination.next;
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
    /// if let Some(episode) = webtoon.episode(1) {
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
        if let Some(page) = &*self.page.read() {
            Ok(page.panels.clone())
        } else {
            let page = self.scrape().await?;
            let panels = page.panels.clone();
            *self.page.write() = Some(page);
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
    /// if let Some(episode) = webtoon.episode(1) {
    ///     println!("thumbnail: {}", episode.thumbnail().await?);
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn thumbnail(&self) -> Result<String, EpisodeError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.thumbnail.to_string())
        } else {
            let page = self.scrape().await?;
            let thumbnail = page.thumbnail.clone();
            *self.page.write() = Some(page);
            Ok(thumbnail.to_string())
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
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(316884, Type::Canvas).await? else {
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
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
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
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
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
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
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
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6679, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
    ///     match episode.post("Loved this episode!", false).await {
    ///         Ok(_) => println!("Left comment on episode {} of {}!", episode.number(), webtoon.title().await?),
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
    /// let Some(webtoon) = client.webtoon(316884, Type::Canvas).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = episodes.episode(1) {
    ///     let panels = episode.download().await?;
    ///     panels.save_single("panels/").await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download(&self) -> Result<Panels, EpisodeError> {
        use tokio::sync::Semaphore;

        let mut panels = if let Some(page) = &*self.page.read() {
            page.panels.clone()
        } else {
            let page = self.scrape().await?;
            let panels = page.panels.clone();
            *self.page.write() = Some(page);
            panels
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
            page: Arc::new(RwLock::new(None)),
            views: None,
            ad_status: None,
            published_status: None,
        }
    }

    /// Scrapes episode page, getting `note`, `length`, `title`, `thumbnail` and the urls for the panels.
    async fn scrape(&self) -> Result<Page, EpisodeError> {
        let response = self
            .webtoon
            .client
            .get_episode(&self.webtoon, self.number)
            .await?;

        if response.status() == 404 {
            return Err(EpisodeError::NotViewable);
        }

        let text = response.text().await?;

        let html = Html::parse_document(&text);

        let page = Page::parse(&html, self.number).context(text)?;

        Ok(page)
    }

    /// Returns `true` id episode exists, `false` if not. Returns `PostError` if there was an error.
    pub(super) async fn exists(&self) -> Result<bool, PostError> {
        posts::check_episode_exists(self).await
    }
}

pub(super) fn season(title: &str) -> Option<u8> {
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
/// for episode in &episodes {
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
