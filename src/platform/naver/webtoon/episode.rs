//! Module containing things related to an episode on `webtoons.com`.

mod page;
pub mod posts;

use chrono::FixedOffset;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::TimeZone;
pub use page::panels::Panel;
pub use page::panels::Panels;

use anyhow::{Context, anyhow};
use chrono::{DateTime, Utc};
use core::fmt;
use parking_lot::RwLock;
use posts::Post;
use regex::Regex;
use scraper::Html;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use self::page::Page;
use self::posts::Posts;
use crate::platform::naver::client::episodes::Article;
use crate::platform::naver::client::episodes::Root;
pub use crate::platform::naver::client::episodes::Sort;
use crate::platform::naver::errors::{EpisodeError, PostError};

use super::Webtoon;

// TODO: With the episode number, might be able to use the episodes api to
// make a single request to get the published date, if it exists.

/// Represents an episode on `comic.naver.com`.
#[derive(Clone)]
pub struct Episode {
    pub(crate) webtoon: Webtoon,
    pub(crate) number: u16,
    pub(crate) title: Arc<RwLock<Option<String>>>,
    pub(crate) thumbnail: Arc<RwLock<Option<String>>>,
    pub(crate) season: Arc<RwLock<Option<u8>>>,
    pub(crate) published: Option<DateTime<Utc>>,
    pub(crate) page: Arc<RwLock<Option<Page>>>,
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
        if let Some(title) = &*self.title.read() {
            Ok(title.clone())
        } else {
            let response = self
                .webtoon
                .client
                .get_episode_info_from_json(&self.webtoon, self.number)
                .await?;

            let txt = response.text().await?;

            let json: Root =
                serde_json::from_str(&txt).map_err(|err| EpisodeError::Unexpected(err.into()))?;

            let episode = json
                .article_list
                .iter()
                .find(|episode| episode.no == self.number)
                .context("episode is known to exist and thus should show up in the episode list")?;

            *self.title.write() = Some(episode.subtitle.clone());

            Ok(episode.subtitle.clone())
        }
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
    /// ```rust
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
        let title = self.title().await?;
        let season = self::season(&title);
        *self.season.write() = season;
        Ok(season)
    }

    /// Returns the creator note for episode.
    pub async fn note(&self) -> Result<Option<String>, EpisodeError> {
        if let Some(page) = &*self.page.read() {
            Ok(page.note.clone())
        } else {
            match self.scrape().await {
                Ok(page) => {
                    let note = page.note.clone();
                    *self.page.write() = Some(page);
                    Ok(note)
                }
                Err(EpisodeError::NotViewable) => Ok(None),
                Err(err) => Err(err),
            }
        }
    }

    /// Returns the sum of the vertical length in pixels.
    pub async fn length(&self) -> Result<Option<u32>, EpisodeError> {
        if let Some(mut panels) = self.panels().await? {
            let mut length = 0;

            for panel in &mut panels {
                panel.download(&self.webtoon.client).await?;
                let image =
                    image::load_from_memory_with_format(&panel.bytes, image::ImageFormat::Jpeg) //
                        .context("invalid image format detected")?;
                length += image.height();
            }

            return Ok(Some(length));
        }

        Ok(None)
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
    /// ```rust
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
    pub async fn published(&self) -> Result<Option<i64>, EpisodeError> {
        if let Some(datetime) = self.published {
            return Ok(Some(datetime.timestamp_millis()));
        }

        let response = self
            .webtoon
            .client
            .get_episode_info_from_json(&self.webtoon, self.number)
            .await?;

        let txt = response.text().await?;

        let json: Root =
            serde_json::from_str(&txt).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let episode = json
            .article_list
            .iter()
            .find(|episode| episode.no == self.number)
            .context("episode is known to exist and thus should show up in the episode list")?;

        Ok(
            published(&episode.service_date_description)
                .map(|datetime| datetime.timestamp_millis()),
        )
    }

    /// Returns the like count for the episode.
    pub async fn likes(&self) -> Result<u32, EpisodeError> {
        use crate::platform::naver::client::likes::Likes;

        let response = self
            .webtoon
            .client
            .get_likes_for_episode(self)
            .await?
            .text()
            .await?;

        let likes = serde_json::from_str::<Likes>(&response).context(response)?;

        Ok(likes.count())
    }

    /// Returns the comment and reply count for the episode.
    ///
    /// Tuple is returned as `(comments, replies)`.
    pub async fn comments_and_replies(&self) -> Result<(u32, u32), PostError> {
        use crate::platform::naver::client::posts::Posts;

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1)
            .await?
            .text()
            .await?;

        let txt = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api = serde_json::from_str::<Posts>(txt).with_context(|| txt.to_string())?;

        let comments = api.result.count.comment;
        let replies = api.result.count.reply;

        Ok((comments, replies))
    }

    // TODO: Rating contains how many people left a rating, expose somehow
    /// Returns the like count for the episode.
    pub async fn rating(&self) -> Result<f64, EpisodeError> {
        use crate::platform::naver::client::episodes::Root;
        use crate::platform::naver::client::rating::Rating;

        let response = self.webtoon.client.get_rating_for_episode(self).await?;

        // As a created `Episode` must always exist, a 404 here means that
        // the episode is not freely public. An extra step is needed to get
        // the rating for an episode behind the cookie system.
        let rating = if response.status() == 404 {
            let response = self
                .webtoon
                .client
                .get_episodes_json(&self.webtoon, 1, Sort::Desc)
                .await?
                .text()
                .await?;

            let episodes: Root =
                serde_json::from_str(&response) //
                    .map_err(|err| EpisodeError::Unexpected(err.into()))?;

            let Some(episode) = episodes
                .charge_folder_article_list
                .iter()
                .find(|episode| episode.no == self.number)
            else {
                return Err(EpisodeError::Unexpected(anyhow!(
                    "episode `{}` wasn't a feely public episode, yet was also not found in the paid episodes list",
                    self.number
                )));
            };

            episode.star_score
        } else {
            serde_json::from_str::<Rating>(&response.text().await?)
                .map_err(|err| EpisodeError::Unexpected(err.into()))?
                .star_info
                .average_star_score
        };

        Ok(rating)
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
    /// ```rust
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
        #[allow(clippy::mutable_key_type)]
        let mut posts = HashSet::new();

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
            .with_context(|| text.to_string())?;

        let pages = api.result.page_model.total_pages;

        // Add first posts
        for post in api.result.comment_list {
            if post.deleted {
                continue;
            }

            posts.insert(Post::try_from((self, post)).with_context(|| text.to_string())?);
        }

        for page in 2..pages {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, page)
                .await?
                .text()
                .await?;

            let text = response
                .trim_start_matches("_callback(")
                .trim_end_matches(");");

            let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
                .with_context(|| text.to_string())?;

            for post in api.result.comment_list {
                if post.deleted {
                    continue;
                }

                posts.insert(Post::try_from((self, post)).with_context(|| text.to_string())?);
            }
        }

        let mut posts = Posts {
            posts: posts.into_iter().collect(),
        };

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
    /// ```rust
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # if let Some(episode) = webtoon.episode(1).await? {
    /// episode.posts_for_each( async |post| {
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
    pub async fn posts_for_each<C: AsyncFn(Post)>(&self, closure: C) -> Result<(), PostError> {
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
            .with_context(|| text.to_string())?;

        let pages = api.result.page_model.total_pages;

        // Add first posts
        for post in api.result.comment_list {
            if post.deleted {
                continue;
            }

            closure(Post::try_from((self, post))?).await;
        }

        for page in 2..pages {
            let response = self
                .webtoon
                .client
                .get_posts_for_episode(self, page)
                .await?
                .text()
                .await?;

            let text = response
                .trim_start_matches("_callback(")
                .trim_end_matches(");");

            let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
                .with_context(|| text.to_string())?;

            for post in api.result.comment_list {
                if post.deleted {
                    continue;
                }

                closure(Post::try_from((self, post))?).await;
            }
        }

        Ok(())
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
    /// ```rust
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
    pub async fn panels(&self) -> Result<Option<Vec<Panel>>, EpisodeError> {
        if let Some(page) = &*self.page.read() {
            Ok(Some(page.panels.clone()))
        } else {
            match self.scrape().await {
                Ok(page) => {
                    let panels = page.panels.clone();
                    *self.page.write() = Some(page);
                    Ok(Some(panels))
                }
                Err(EpisodeError::NotViewable) => Ok(None),
                Err(err) => Err(err),
            }
        }
    }

    /// Returns the thumbnail URL for episode.
    pub async fn thumbnail(&self) -> Result<String, EpisodeError> {
        if let Some(thumbnail) = &*self.thumbnail.read() {
            Ok(thumbnail.to_string())
        } else {
            let response = self
                .webtoon
                .client
                .get_episode_info_from_json(&self.webtoon, self.number)
                .await?;

            let txt = response.text().await?;

            let json: Root =
                serde_json::from_str(&txt).map_err(|err| EpisodeError::Unexpected(err.into()))?;

            let episode = json
                .article_list
                .iter()
                .find(|episode| episode.no == self.number)
                .context("episode is known to exist and thus should show up in the episode list")?;

            *self.thumbnail.write() = Some(episode.thumbnail_url.clone());

            Ok(episode.thumbnail_url.clone())
        }
    }

    /// Will download the panels of episode.
    ///
    /// This returns a [`Panels`] which offers ways to save to disk.
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

            let image = image::load_from_memory(&panel.bytes)
                .map_err(|err| EpisodeError::Unexpected(err.into()))?;

            height += image.height();
            width = width.max(image.width());
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
    /// ```rust
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
        *self.page.write() = None;
    }
}

// Internal use only
impl Episode {
    pub(crate) fn new(webtoon: &Webtoon, number: u16) -> Self {
        Self {
            webtoon: webtoon.clone(),
            number,
            title: Arc::new(RwLock::new(None)),
            season: Arc::new(RwLock::new(None)),
            thumbnail: Arc::new(RwLock::new(None)),
            // NOTE: Currently there is no way to get this info from an episodes page.
            // The only sources are the dashboard episode list data, and the episode list from the webtoons page.
            // This could be gotten, in theory, with the webtoons page episode data, but caching the episodes
            // would lead to a large refactor and be slow for when only getting one episodes data.
            // For now will just return None until a solution can be landed on.
            published: None,
            page: Arc::new(RwLock::new(None)),
        }
    }

    /// Scrapes episode page, getting `note`, `length`, `title`, `thumbnail` and the urls for the panels.
    async fn scrape(&self) -> Result<Page, EpisodeError> {
        let response = self
            .webtoon
            .client
            .get_episode_page_html(&self.webtoon, self.number)
            .await?;

        if response.status() == 302 {
            return Err(EpisodeError::NotViewable);
        }

        let text = response.text().await?;
        let html = Html::parse_document(&text);
        let page = Page::parse(&text, &html, self.number).context(text)?;

        Ok(page)
    }

    /// Returns `true` id episode exists, `false` if not. Returns `EpisodeError` if there was an error.
    pub(super) async fn exists(&self) -> Result<bool, EpisodeError> {
        use crate::platform::naver::client::episodes::Root;

        let response = self
            .webtoon
            .client
            .get_episodes_json(&self.webtoon, 1, Sort::Desc)
            .await?
            .text()
            .await?;

        let episodes: Root = serde_json::from_str(&response) //
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let max = if let Some(episode) = episodes.charge_folder_article_list.first() {
            episode.no
        } else if let Some(episode) = episodes.article_list.first() {
            episode.no
        } else {
            return Ok(false);
        };

        Ok(self.number <= max)
    }
}

impl From<(Webtoon, Article)> for Episode {
    fn from((webtoon, article): (Webtoon, Article)) -> Self {
        Self {
            webtoon,
            number: article.no,
            season: Arc::new(RwLock::new(season(&article.subtitle))),
            title: Arc::new(RwLock::new(Some(article.subtitle))),
            thumbnail: Arc::new(RwLock::new(Some(article.thumbnail_url))),
            published: published(&article.service_date_description),
            page: Arc::new(RwLock::new(None)),
        }
    }
}

pub(super) fn season(title: &str) -> Option<u8> {
    // Tower of God: 2부 103화
    let part_digit = Regex::new(r"(?P<season>\d+)부 ").expect("regex should be valid");

    if let Some(capture) = part_digit.captures(title.as_ref()) {
        let season = capture["season"]
            .parse::<u8>()
            .expect(r"regex match on `\d+` so should be parsable as an int");

        return Some(season);
    }

    None
}

fn published(date: &str) -> Option<DateTime<Utc>> {
    let mut date = date.split('.');

    let year: i32 = date.next()?.parse().ok()?;
    let month: u32 = date.next()?.parse().ok()?;
    let day: u32 = date.next()?.parse().ok()?;

    // `year` is only a year in the century 2000, so need to add 2000: 25 + 2000 = 2025
    let naive = NaiveDate::from_ymd_opt(year + 2000, month, day)?;
    let naive_time = NaiveDateTime::new(naive, NaiveTime::from_hms_opt(0, 0, 0)?);

    let offset = FixedOffset::east_opt(9 * 3600)?; // UTC + 9
    let date = offset.from_local_datetime(&naive_time).single()?;

    Some(date.into())
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
#[derive(Debug, Clone)]
pub struct Episodes {
    pub(crate) count: u16,
    pub(crate) episodes: Arc<[Episode]>,
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
            episodes: value.into(),
        }
    }
}

impl<'a> IntoIterator for &'a Episodes {
    type Item = &'a Episode;

    type IntoIter = std::slice::Iter<'a, Episode>;

    fn into_iter(self) -> Self::IntoIter {
        self.episodes.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_proper_date() {
        let datetime = published("25.05.05").unwrap();
        let expected =
            DateTime::parse_from_str("25.05.05 00:00:00 +0900", "%y.%m.%d %H:%M:%S %z").unwrap();
        eprintln!("{datetime}");
        assert_eq!(expected, datetime);
    }
}
