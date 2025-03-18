//! Module containing things related to an episode on `webtoons.com`.

mod page;
pub mod posts;

pub use page::panels::Panel;
#[cfg(feature = "download")]
pub use page::panels::Panels;

use anyhow::Context;
use chrono::{DateTime, Utc};
use core::fmt;
use posts::Post;
use regex::Regex;
use scraper::Html;
use serde_json::json;
use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use std::{hash::Hash, str::FromStr};
use tokio::sync::Mutex;

use self::page::Page;
use self::posts::Posts;
use crate::platform::webtoons::client::likes::Likes;
use crate::platform::webtoons::client::posts::id::Id;
use crate::platform::webtoons::client::posts::PostsResult;
use crate::platform::webtoons::{
    errors::{ClientError, EpisodeError, PostError},
    meta::Scope,
};

use super::{dashboard::episodes::DashboardStatus, Webtoon};

// TODO: `episode.post_with_sticker("POST", false, Sticker::from_str("")?)`
// TODO: `episode.post_with_webtoons("POST", false, vec![Webtoon])`
// TODO: `episode.post_with_giphy("POST", false, Giphy::new(String::from("")))`
//
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
#[derive(Clone)]
pub struct Episode {
    pub(crate) webtoon: Webtoon,
    pub(crate) number: u16,
    pub(crate) season: Arc<Mutex<Option<u8>>>,
    pub(crate) title: Arc<Mutex<Option<String>>>,
    pub(crate) published: Option<DateTime<Utc>>,
    pub(crate) page: Arc<Mutex<Option<Page>>>,
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
    /// This matches up with the `episode_no=` URL query. This does not necessarily match up with the `#NUMBER` on the episode list.
    #[must_use]
    pub const fn number(&self) -> u16 {
        self.number
    }

    /// Returns the title of the episode.
    pub async fn title(&self) -> Result<String, EpisodeError> {
        let mut title = self.title.lock().await;

        if title.is_none() {
            let mut page = self.page.lock().await;

            if page.is_none() {
                *page = Some(self.scrape().await?);
            }

            *title = Some(
                page.as_ref()
                    .context(
                        "page should have been scraped with `self.scrape` and so should be `Some`",
                    )?
                    .title
                    .clone(),
            );
        }

        Ok(title
            .as_ref()
            .context("title should have been scraped from prior Page scrape")?
            .clone())
    }

    /// Returns the season number if a pattern is detected in the episode's title.
    ///
    /// The method attempts to extract the season number by searching for specific patterns within the episode's title.
    /// The supported patterns are:
    /// - `[Season \d+]`
    /// - `(Season \d+)`
    /// - `[S\d+]`
    /// - `(S\d+)`
    ///
    /// If no season pattern is found, the method will return `None`.
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// let season_number = episode.season().await?;
    /// if let Some(season) = season_number {
    ///     println!("Season: {}", season);
    /// } else {
    ///     println!("No season detected.");
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors:
    /// - Returns an [`EpisodeError`] if an error occurs during the retrieval of the episode title or unexpected issues occur.
    pub async fn season(&self) -> Result<Option<u8>, EpisodeError> {
        let mut season = self.season.lock().await;

        if season.is_none() {
            let title = self.title().await?;
            *season = self::season(&title);
        }

        Ok(*season)
    }

    /// Returns the creator note for episode.
    pub async fn note(&self) -> Result<Option<String>, EpisodeError> {
        let mut page = self.page.lock().await;

        if page.is_none() {
            *page = Some(self.scrape().await?);
        }

        let note = page
            .as_ref()
            .map(|page| page.note.clone())
            .context("episode `page` should have been updated with the call to `self.scrape`")?;

        drop(page);

        Ok(note)
    }

    /// Returns the sum of the vertical length in pixels.
    pub async fn length(&self) -> Result<u32, EpisodeError> {
        let mut page = self.page.lock().await;

        if page.is_none() {
            *page = Some(self.scrape().await?);
        }

        let length = page
            .as_ref()
            .context("episode `page` should have been updated with the call to `self.scrape`")?
            .length;

        drop(page);

        Ok(length)
    }

    /// Returns the published timestamp of the episode.
    ///
    /// It returns as [`Some(i64)`] if the episode is publicly available or has a set publish date.
    /// Otherwise, it returns [`None`] if the episode is unpublished.
    ///
    /// ### Behavior
    ///
    /// - **Original vs. Canvas Episodes**:
    ///   - **Original Webtoons**: For episodes from an Original series, this method will always return [`Some(i64)`] since Originals follow a standard publishing schedule.
    ///   - **Canvas Webtoons (No Session)**: For Canvas episodes, if no session is provided to the [`Client`](super::Client), it will also return [`Some(i64)`], reflecting publicly available information.
    ///   - **Canvas Webtoons (With Session)**: If a valid creator session is provided for a Canvas webtoon, it may return [`None`] if the episode is unpublished (i.e., still in draft form).
    ///
    /// - **Important Caveat**:
    ///   - This method **only returns a value** when the episode is accessed via the `webtoon.episodes()` method, which retrieves all episodes, including unpublished ones when available. If the episode is retrieved using `webtoon.episode(N)`, this method will always return [`None`], even if the episode is published.
    ///   - Using `webtoon.episodes()` ensures that published episodes return accurate timestamps. For episodes retrieved without a valid creator session, the published time will be available but may default to **2:00 AM** on the publication date due to webtoon page limitations.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error,webtoon::episode::PublishedStatus};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// if let Some(published) = episode.published() {
    ///     println!("Episode was published on: {}", published);
    /// } else {
    ///     println!("Episode is unpublished or the published date is unavailable.");
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Notes
    ///
    /// - The published date is available for public and Original episodes, but episodes behind fast-pass or ad walls, or those in draft form, may return [`None`].
    /// - To get accurate publishing times for episodes in draft or restricted access (fast-pass/ad), the [`Client`](super::Client) session must belong to the webtoon creator.
    /// - **Reminder**: `published()` will always return [`None`] if used with `webtoon.episode(N)`; use `webtoon.episodes()` for access to the publication date.
    #[must_use]
    pub fn published(&self) -> Option<i64> {
        self.published.map(|datetime| datetime.timestamp_millis())
    }

    /// Returns the view count for the episode as `Some(u32)` if available, or `None` if the view count is not accessible.
    ///
    /// ### Behavior
    ///
    /// - **Original vs. Canvas Episodes**:
    ///   - **Original Webtoons**: For episodes from an Original series will always return `None`.
    ///   - **Canvas Webtoons (No Session)**: Will always return `None`.
    ///   - **Canvas Webtoons (With Session)**: If a valid creator session is provided, the method will include views for all episodes, including those behind fast-pass, ad walls, or even unpublished episodes, provided the episode is retrieved using `webtoon.episodes()`.
    ///
    /// - **Important Caveat**:
    ///   - **Views will always return [`None`]** when using the `webtoon.episode(N)` method to retrieve a single episode. To get the view count, you **must use `webtoon.episodes()`**, which fetches all episodes in bulk and provides view count data when available.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// if let Some(views) = episode.views() {
    ///     println!("This episode has {} views.", views);
    /// } else {
    ///     println!("View count is unavailable for this episode.");
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Notes
    ///
    /// - View counts for episodes behind fast-pass, ad walls, or unpublished drafts are only available when the session belongs to the creator.
    /// - If the episode is accessed using `webtoon.episode(N)`, the view count will always return [`None`].
    #[must_use]
    pub fn views(&self) -> Option<u32> {
        self.views
    }

    /// Returns the like count for the episode.
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
    /// ### Behavior
    ///
    /// - **Fetching Comments**:
    ///   - The method ensures no duplicates are returned, even if paginated results overlap.
    ///   - The comments are returned in order from **newest to oldest**.
    ///   - Direct replies that have been deleted (but have replies) will still be included with a message indicating the deletion. Comments deleted without replies will not be included.
    ///
    /// ### Caveat
    ///
    /// - The method retrieves only **direct** (top-level) posts. Replies to these posts (nested replies) are not included.
    /// - The behavior remains consistent for episodes accessed through either `webtoon.episodes()` or `webtoon.episode(N)`.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// let posts = episode.posts().await?;
    /// for post in posts {
    ///     println!("Comment by {}: {}", post.poster().username(), post.body().contents());
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - Returns a [`PostError`] if there is an issue with the client or an unexpected error occurs during the post retrieval process.
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

        let url = format!("https://www.webtoons.com/p/api/community/v1/page/{page_id}/posts/search?pinRepresentation=distinct&prevSize=0&nextSize=1");

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

    /// Iterates over all direct (top-level) comments for the episode and applies a callback function to each post, without storing them in memory.
    ///
    /// This method is useful in scenarios where memory constraints are an issue, as it avoids loading all posts into memory at once. Instead, each post is processed immediately as it is retrieved, making it more memory-efficient than the `posts()` method.
    ///
    /// ### Behavior
    ///
    /// - **Memory Efficiency**:
    ///   - Unlike `posts()`, which retrieves and stores all posts in memory before returning them, this method processes each post immediately using the provided callback function.
    ///   - Ideal for environments with limited memory, as it avoids the need to load all posts simultaneously.
    ///
    /// - **Direct (Top-level) Posts**:
    ///   - Retrieves only direct (top-level) posts, not nested replies.
    ///   - Posts are fetched and processed in batches, but publish order is not guaranteed due to the limitations of the API.
    ///
    /// ### Limitations
    ///
    /// - **Duplicate Posts**:
    ///   - Due to potential API inconsistencies during pagination, this method cannot guarantee that duplicate posts will be filtered out.
    ///   
    /// - **Publish Order**:
    ///   - The order in which posts are published may not be respected, as the posts are fetched and processed in batches that may appear out of order.
    ///
    /// ### Parameters
    ///
    /// - `callback`: A function or closure that takes a `Post` and processes it asynchronously. It must return a `Future` that completes with `()` (unit type).
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.posts_for_each(|post| async move {
    ///     println!("Processing comment: {}", post.body().contents());
    /// }).await?;
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - Returns a [`PostError`] if there is an issue with the client or an error occurs during the retrieval of posts.
    ///
    /// ### Usage Consideration
    ///
    /// If your application has limited memory and the collection of all posts at once is not feasible, this method provides a better alternative. However, consider the trade-offs, such as possible duplicates and lack of guaranteed publish order.
    pub async fn posts_for_each<F, Fut>(&self, callback: F) -> Result<(), PostError>
    where
        F: Fn(Post) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        // Adds `is_top/isPinned` info. The previous API loses this info but is easier to work with so
        // This extra step to the other API is a one off to get only the top comment info attached to
        // the top 3 posts.
        let page_id = format!(
            "{}_{}_{}",
            self.webtoon.scope.as_single_letter(),
            self.webtoon.id,
            self.number
        );

        let url = format!("https://www.webtoons.com/p/api/community/v1/page/{page_id}/posts/search?pinRepresentation=distinct&prevSize=0&nextSize=1");

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
                callback(Post::try_from((self, post))?).await;
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
            callback(Post::try_from((self, post))?).await;
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
                callback(Post::try_from((self, post))?).await;
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
    /// If the post with the `id` was deleted or missing, this method may perform a full scan of the episode.
    /// For cases where the post's date is known but the `id` is uncertain to exist, use [`Self::posts_till_date`], although you may need
    /// to handle potential duplicates.
    ///
    /// ### Behavior
    ///
    /// - **Fetching Comments**:
    ///   - Retrieves comments from newest to oldest, scanning each batch of posts until it encounters the post with the provided `id`.
    ///   - If the post with the given `id` is found, the method returns all posts up to, not including, the post with the passed in `id`.
    ///   - If the post is not found, the method continues to fetch subsequent pages of comments until it either finds the post or reaches the end of the episode's comments.
    ///   - **Deleted Posts**: If a post with the `id` has no replies and is deleted, it is not returned in the response, which can lead to a full scan of the episode's posts before realizing the post is not present.
    ///
    /// ### Caveats
    ///
    /// - **Deleted Posts**: Posts without replies that have been deleted are not returned in the results, which may lead to a situation where the post with the given `id` is never found, causing the method to scan the entire episode.
    /// - **Performance Consideration**: If the post is near the end of the episode's comments, the method may need to scan all the way through to find the `id`, which can impact performance for episodes with large comment sections.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error,webtoon::episode::PublishedStatus};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// let posts = episode.posts_till_id("some-post-id").await?;
    /// for post in posts {
    ///     println!("Comment by {}: {}", post.poster().username(), post.body().contents());
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - Returns a [`PostError`] if there is an issue with the client or an unexpected error occurs during the post retrieval process.
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
    /// ### Behavior
    ///
    /// - **Fetching Comments**:
    ///   - Retrieves comments from newest to oldest, scanning each batch until it encounters a post older than the given `date`.
    ///   - If a post has the exact timestamp as the provided `date`, the method continues scanning to ensure all posts with the same timestamp are fetched. This ensures no posts are missed due to duplicate timestamps.
    ///   - If a post's creation date is older than the specified `date`, the method returns all posts encountered up to that point.
    ///
    /// ### Caveats
    ///
    /// - **Duplicate Timestamps**: Posts can have the same creation date, so the method ensures that all posts with a given timestamp are returned before stopping.
    /// - **Performance**: Similar to `posts_till_id`, this method may impact performance for episodes with many comments, especially if the `date` is far in the past.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error,webtoon::episode::PublishedStatus};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// let posts = episode.posts_till_date(1729582054).await?;
    /// for post in posts {
    ///     println!("Comment by {}: {}", post.poster().username(), post.body().contents());
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - Returns a [`PostError`] if there is an issue with the client or an unexpected error occurs during the post retrieval process.
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
    /// This method retrieves the panels associated with the episode. The data is cached for performance, so subsequent calls will not trigger a refetch unless explicitly evicted.
    ///
    /// ### Behavior
    ///
    /// - **Caching**:
    ///   - The panels are cached after the first retrieval for performance reasons. If the cache is present, the method will return the cached data. If you need to force a refetch of the episode's data, consider using `evict_cache()`.
    ///
    /// - **Panel URLs**:
    ///   - Each panel's URL can be accessed using the returned list of panels. This is useful for downloading or viewing individual panels.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error, webtoon::episode::PublishedStatus};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// for panel in episode.panels().await? {
    ///     println!("url: {}", panel.url());
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - Returns an [`EpisodeError`] if there is a failure in fetching or processing the episode data.
    pub async fn panels(&self) -> Result<Vec<Panel>, EpisodeError> {
        let mut page = self.page.lock().await;

        if page.is_none() {
            *page = Some(self.scrape().await?);
        }

        let panels = page
            .as_ref()
            .context("episode `page` should have been updated with the call to `self.scrape")?
            .panels
            .clone();

        drop(page);

        Ok(panels)
    }

    /// Returns the thumbnail URL for episode.
    pub async fn thumbnail(&self) -> Result<String, EpisodeError> {
        let mut page = self.page.lock().await;

        if page.is_none() {
            *page = Some(self.scrape().await?);
        }

        let thumbnail = page
            .as_ref()
            .context("episode `page` should have been updated with the call to `self.scrape")?
            .thumbnail
            .as_str()
            .to_string();

        drop(page);

        Ok(thumbnail)
    }

    /// Returns the [`PublishedStatus`] for the episode, indicating whether the episode is published, a draft, or removed.
    ///
    /// This can only be accurately determined when using the `Webtoon::episodes` method. If the `Webtoon::episode` method is used,
    /// this function will always return `None`, as the necessary metadata to determine an episode's published status is not available
    /// through `Webtoon::episode`.
    ///
    /// The possible states are:
    /// - [`PublishedStatus::Published`] - The episode is publicly available in some capacity (ad, fast-pass, or fully public).
    /// - [`PublishedStatus::Draft`] - The episode is not published in any form yet (it is in draft status).
    /// - [`PublishedStatus::Removed`] - The episode has been removed from publication.
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error,webtoon::episode::PublishedStatus};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// if let Some(status) = episode.published_status() {
    ///     match status {
    ///         PublishedStatus::Published => println!("Episode is published."),
    ///         PublishedStatus::Draft => println!("Episode is still a draft."),
    ///         PublishedStatus::Removed => println!("Episode has been removed."),
    ///     }
    /// } else {
    ///     println!("Unable to determine published status.");
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Caveat:
    /// - This method relies on full episode data, which is only accessible via `Webtoon::episodes`. When using `Webtoon::episode`, the metadata provided is insufficient for determining the publication status.
    #[must_use]
    pub fn published_status(&self) -> Option<PublishedStatus> {
        self.published_status
    }

    /// Returns the episode's current ad status.
    ///
    /// This information is only available if a session is provided, and the webtoon in question was created by the user of the session.
    /// In any other scenario, this method returns `None`. To retrieve this data, the `episodes` function must be used when getting episode data.
    ///
    /// ### Original Series:
    /// - For original webtoons, it's not possible to determine the ad status from the public episode listing alone.
    ///   - Generally, any random original episode may have been behind fast-pass, but initial release episodes (which are typically not behind fast-pass) would be indistinguishable.
    ///   - Therefore, for original series, this method will always return `None`.
    ///
    /// ### Canvas Series:
    /// - For canvas webtoons created by the session's user, the ad status can be retrieved and returned if applicable.
    /// - If no session is provided and only public info is used, this will always return `None`.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// let ad_status = episode.ad_status();
    /// if let Some(status) = ad_status {
    ///     println!("Ad status: {:?}", status);
    /// } else {
    ///     println!("Ad status is unavailable or not applicable.");
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Caveats:
    /// - **Requires Session**: The method only works if the episode belongs to a canvas series created by the session’s user.
    /// - **Original Series Limitation**: For original series, ad status will always return `None` due to the limitations of publicly available information.
    #[must_use]
    pub fn ad_status(&self) -> Option<AdStatus> {
        self.ad_status
    }

    /// Likes the episode on behalf of the user associated with the current session.
    ///
    /// This method allows the user (via their session) to like a specific episode. If no session is present or invalid, it will return an [`EpisodeError`].
    /// If episode is liked, it will do nothing.
    ///
    /// ### Behavior:
    /// - **Session Required**: The method will attempt to like the episode on behalf of the user tied to the current session.
    /// - **Webtoon Ownership**: If the episode belongs to the current user’s own webtoon, it will still process the request without issue.
    /// - **Errors**:
    ///   - If no session is available, or the session is invalid, it will return an [`EpisodeError::ClientError`].
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.like().await?;
    /// println!("Episode liked successfully!");
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors:
    /// - Returns an [`EpisodeError`] if an error occurs during the process, including invalid session or unexpected client errors.
    pub async fn like(&self) -> Result<(), EpisodeError> {
        self.webtoon.client.like_episode(self).await?;
        Ok(())
    }

    /// Removes the like for the episode on behalf of the user associated with the current session.
    ///
    /// This method allows the user (via their session) to remove a like from a specific episode. If no session is present or invalid, it will return an [`EpisodeError`].
    /// If episode is not liked, it will do nothing.
    ///
    /// ### Behavior:
    /// - **Session Required**: The method will attempt to remove the like from the episode on behalf of the user tied to the current session.
    /// - **Webtoon Ownership**: If the episode belongs to the current user’s own webtoon, the request will process without issue.
    /// - **Errors**:
    ///   - If no session is available, or the session is invalid, it will return an [`EpisodeError::ClientError`].
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.unlike().await?;
    /// println!("Like removed from episode successfully!");
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors:
    /// - Returns an [`EpisodeError`] if an error occurs during the process, such as an invalid session or unexpected client errors.
    pub async fn unlike(&self) -> Result<(), EpisodeError> {
        self.webtoon.client.unlike_episode(self).await?;
        Ok(())
    }

    /// Posts a top-level comment on the episode.
    ///
    /// This method allows users to leave a comment on an episode. The comment can be marked as a spoiler.
    ///
    /// ### Parameters:
    /// - `body`: The content of the comment to be posted.
    /// - `is_spoiler`: A boolean indicating whether the comment should be marked as a spoiler. If `true`, the comment will be marked as a spoiler.
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.post("Loved this episode!", false).await?;
    /// episode.post("Shocking twist! *spoiler*", true).await?;
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors:
    /// - Returns a [`PostError`] if there is an issue during the post request, such as a missing session, invalid token, or server error.
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
    /// This returns a [`Panels`] which offers ways to save to disk.
    #[cfg(feature = "download")]
    pub async fn download(&self) -> Result<Panels, EpisodeError> {
        use tokio::sync::Semaphore;

        let mut page = self.page.lock().await;
        if page.is_none() {
            *page = Some(self.scrape().await?);
        }

        let mut panels = page
            .as_ref()
            .context("`panel_urls` should be `Some` if scrape succeeded")?
            .panels
            .clone();

        drop(page);

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

    /// Evicts the cached episode page, forcing a refetch on the next access.
    ///
    /// This method clears the cached episode metadata, such as the episode's title, length, creator note, and other information,
    /// which is stored to improve performance. If the episode data needs to be refreshed or re-fetched (e.g., if updates to the episode occurred),
    /// calling this method ensures that the cache is cleared and the next access will trigger a fresh network request.
    ///
    /// ### Example:
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.evict_cache().await;
    /// let fresh_episode_data = episode.title().await?; // Forces a refetch
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Notes:
    /// - The cache is automatically populated when episode metadata is fetched. Use this method only if you want to invalidate that cache.
    pub async fn evict_cache(&self) {
        let mut page = self.page.lock().await;
        *page = None;
    }
}

// Internal use only
impl Episode {
    pub(crate) fn new(webtoon: &Webtoon, number: u16) -> Self {
        Self {
            webtoon: webtoon.clone(),
            number,
            season: Arc::new(Mutex::new(None)),
            title: Arc::new(Mutex::new(None)),
            // NOTE: Currently there is no way to get this info from an episodes page.
            // The only sources are the dashboard episode list data, and the episode list from the webtoons page.
            // This could be gotten, in theory, with the webtoons page episode data, but caching the episodes
            // would lead to a large refactor and be slow for when only getting one episodes data.
            // For now will just return None until a solution can be landed on.
            published: None,
            page: Arc::new(Mutex::new(None)),
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

        if response.status() == 429 {
            return Err(EpisodeError::ClientError(ClientError::RateLimitExceeded));
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
/// This is a wrapper around a `Vec<Episode>` meant to provide methods for common interactions.
#[derive(Debug)]
pub struct Episodes {
    pub(crate) count: u16,
    pub(crate) episodes: Vec<Episode>,
}

impl Episodes {
    /// Returns the count of the episodes retrieved.
    #[must_use]
    pub fn count(&self) -> u16 {
        self.count
    }

    /// Gets the episode from passed in value if it exists.
    #[must_use]
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
    ///   The episode is available to the public. This includes episodes behind ad or fast-pass paywalls.
    Published,
    ///   The episode is not yet published in any capacity. This means it hasn't been made available to the public or
    ///   put behind ad/fast-pass options.
    Draft,
    ///   The episode was previously published but has since been removed. This might happen due to takedowns, content issues, or other reasons.
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
