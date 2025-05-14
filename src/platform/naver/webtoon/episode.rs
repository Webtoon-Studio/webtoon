//! Module containing things related to an episode on `comic.naver.com`.

mod page;
pub mod posts;

use anyhow::{Context, anyhow};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use core::fmt;
use parking_lot::RwLock;
use regex::Regex;
use scraper::Html;
use std::hash::Hash;
use std::sync::Arc;
use std::{cmp::Ordering, collections::HashSet};

use self::page::Page;
use self::posts::Posts;
use crate::platform::naver::client::episodes::Sort;
use crate::platform::naver::client::{
    self,
    episodes::{Article, Root},
};
use crate::platform::naver::errors::{EpisodeError, PostError};
pub use page::panels::{Panel, Panels};
use posts::Post;

use super::Webtoon;

/// Represents an episode on `comic.naver.com`.
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
/// # use webtoon::platform::naver::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(813443).await? else {
///     unreachable!("webtoon is known to exist");
/// };
///
/// if let Some(episode) = webtoon.episode(50).await? {
///     assert_eq!("50화", episode.title().await?);
/// }
/// # Ok(())
/// # }
/// ```
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
            .field("thumbnail", &self.title)
            .field("page", &self.page)
            .finish()
    }
}

impl Episode {
    /// Returns the episode number.
    ///
    /// This matches up with the `no=` URL query: [`https://comic.naver.com/webtoon/detail?titleId=813443&no=50`]
    ///
    /// Distinctly, this could be different from expectations just basing off of the shown episode numbers, as there could
    /// have been episodes deleted that would shift the numbers.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     assert_eq!(50, episode.number());
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`https://comic.naver.com/webtoon/detail?titleId=813443&no=50`]: https://comic.naver.com/webtoon/detail?titleId=813443&no=50
    #[inline]
    pub fn number(&self) -> u16 {
        self.number
    }

    // TODO: See if deleted/hidden episodes return a title

    /// Returns the title of the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(819946).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     assert_eq!("50화", episode.title().await?);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the season number of the episode.
    ///
    /// The method attempts to extract the season number by searching for specific patterns within the episode's title.
    ///
    /// The supported patterns are:
    /// - `\d+부`
    ///
    /// If no season pattern is found, the method will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(183559).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(650).await? {
    ///     assert_eq!(Some(3), episode.season().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn season(&self) -> Result<Option<u8>, EpisodeError> {
        let title = self.title().await?;
        let season = self::season(&title);
        *self.season.write() = season;
        Ok(season)
    }

    /// Returns the creator note for episode, if it exits, and the episode is publicly viewable.
    ///
    /// Will return `None` if the episode is not publicly viewable. If there is no note, this will also return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(183559).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     assert_eq!("일단 자야겠어요.", episode.note().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the sum of the vertical length in pixels, if publicly and freely viewable.
    ///
    /// If episode is publicly and freely viewable, this will always return `Some` and the value will be greater than `0`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(808482).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     assert_eq!(Some(100000), episode.length().await?);
    /// }
    /// # Ok(())
    /// #}
    /// ```
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
    /// It returns `Some` if the episode is publicly and freely available, otherwise it returns `None`. This date is from
    /// when it becomes freely available, **NOT** when its published behind a paywall or when it was drafted.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     assert_eq!(Some(100000000), episode.published().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the like count for the `Episode`.
    ///
    /// More specifically, it's the number corresponding to `좋아요`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("episode `{}` for `{}` has `{}` likes", episode.number(), webtoon.title(), episode.likes().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the comment and reply count for the `Episode`.
    ///
    /// Tuple is returned as `(comments, replies)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let (comments, replies) = episode.likes().await?;
    ///
    ///     println!("episode `{}` for `{}` has `{comments}` comments and `{replies}` replies", episode.number(), webtoon.title());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn comments_and_replies(&self) -> Result<(u32, u32), PostError> {
        use crate::platform::naver::client::posts::Posts;

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1, 15, client::posts::Sort::Best)
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

    /// Returns the rating of the `Episode`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("episode `{}` for `{}` has a rating of `{}`", episode.number(), webtoon.title(), episode.rating().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rating(&self) -> Result<f64, EpisodeError> {
        self.rating_impl().await.map(|info| info.0)
    }

    /// Returns the number of people who left a rating on the episode.
    ///
    /// # Returns
    ///
    /// This will return `None` if the episode is non-free. For any public free episode, this should always return `Some`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("episode `{}` for `{}` had `{:?}` people leave a rating", episode.number(), webtoon.title(), episode.raters().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn raters(&self) -> Result<Option<u32>, EpisodeError> {
        self.rating_impl().await.map(|info| info.1)
    }

    /// Retrieves all direct (top-level) comments for the episode.
    ///
    /// There are no duplicate comments, and only direct comments (top-level) are fetched, not the nested replies.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     for post in episode.posts().await? {
    ///        println!("{}: {}", post.poster().username(), post.body());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts(&self) -> Result<Posts, PostError> {
        #[allow(clippy::mutable_key_type)]
        let mut posts = HashSet::new();

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1, 100, client::posts::Sort::New)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api =
            serde_json::from_str::<client::posts::Posts>(text).with_context(|| text.to_string())?;

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
                .get_posts_for_episode(self, page, 100, client::posts::Sort::New)
                .await?
                .text()
                .await?;

            let text = response
                .trim_start_matches("_callback(")
                .trim_end_matches(");");

            let api = serde_json::from_str::<client::posts::Posts>(text)
                .with_context(|| text.to_string())?;

            for post in api.result.comment_list {
                if post.deleted {
                    continue;
                }

                posts.insert(Post::try_from((self, post)).with_context(|| text.to_string())?);
            }
        }

        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1, 15, client::posts::Sort::New)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api =
            serde_json::from_str::<client::posts::Posts>(text).with_context(|| text.to_string())?;

        // Add `is_top` info to posts
        for post in api.result.comment_list {
            if post.deleted {
                continue;
            }

            posts.replace(Post::try_from((self, post)).with_context(|| text.to_string())?);
        }

        let posts = Posts {
            posts: posts.into_iter().collect(),
        };

        Ok(posts)
    }

    /// Iterates over all direct (top-level) comments for the episode, applying a callback function to each post.
    ///
    /// This method is useful in scenarios where memory constraints are an issue, as it avoids loading all posts into
    /// memory at once. Instead, each post is processed immediately as it is retrieved, making it more memory-efficient
    /// than the [`posts()`](Episode::posts()) method.
    ///
    /// # Limitations
    ///
    /// - **Duplicate Posts**:
    ///   - Due to potential API inconsistencies during pagination, this method cannot guarantee that duplicate posts will be filtered out.
    ///
    /// - **Publish Order**:
    ///   - The order in which posts are published may not be respected, as the posts are fetched and processed in batches that may appear out of order.
    ///
    /// - **Lacking Some Info**
    ///   - This will always result in [`is_top()`](Post::is_top()) being `false`.
    ///
    /// # Usage Consideration
    ///
    /// If your application has limited memory and the collection of all posts at once is not feasible, this method provides a better alternative.
    /// However, consider the trade-offs, such as possible duplicates and lack of guaranteed publish order.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// }
    ///
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     episode.posts_for_each(async |post| {
    ///         println!("{}: {}", post.poster().username(), post.body());
    ///     }).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn posts_for_each<C: AsyncFn(Post)>(&self, closure: C) -> Result<(), PostError> {
        let response = self
            .webtoon
            .client
            .get_posts_for_episode(self, 1, 100, client::posts::Sort::New)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api =
            serde_json::from_str::<client::posts::Posts>(text).with_context(|| text.to_string())?;

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
                .get_posts_for_episode(self, page, 100, client::posts::Sort::New)
                .await?
                .text()
                .await?;

            let text = response
                .trim_start_matches("_callback(")
                .trim_end_matches(");");

            let api = serde_json::from_str::<client::posts::Posts>(text)
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
    /// If the episode is not freely viewable, this will return `None`. If `Some` is returned, there will always be at
    /// least one panel, as this is required by the platform to publish an episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(836382).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     if let Some(panels) = webtoon.panels().await? {
    ///         for panel in panels {
    ///             println!("url: {}", panel.url());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("thumbnail url: {}". episode.thumbnail().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// This returns a [`Panels`] which offers ways to save panels to disk.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(826341).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let panels = episode.download().await?;
    ///     // For more info, see `Panels` documentation
    ///     panels.save_single("path/to/save/").await?;
    /// }
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
            published: None,
            page: Arc::new(RwLock::new(None)),
        }
    }

    async fn rating_impl(&self) -> Result<(f64, Option<u32>), EpisodeError> {
        use crate::platform::naver::client::episodes::Root;
        use crate::platform::naver::client::rating::Rating;

        let response = self.webtoon.client.get_rating_for_episode(self).await?;

        // As a created `Episode` must always exist, a 404 here means that
        // the episode is not freely public. An extra step is needed to get
        // the rating for an episode behind the cookie system.
        let (rating, raters) = if response.status() == 404 {
            let response = self
                .webtoon
                .client
                // `Desc` as this is known to be a cookie episode.
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

            // This way of getting the rating doesn't not include the amount of people who left a rating.
            (episode.star_score, None)
        } else {
            let info = serde_json::from_str::<Rating>(&response.text().await?)
                .map_err(|err| EpisodeError::Unexpected(err.into()))?
                .star_info;

            (info.average_star_score, Some(info.star_score_count))
        };

        Ok((rating, raters))
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

impl PartialOrd for Episode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Episode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

/// Represents a collection of episodes.
///
/// This type is never constructed, only gotten via [`Webtoon::episodes()`].
///
/// # Example
///
/// ```
/// # use webtoon::platform::naver::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(836382).await? else {
///     unreachable!("webtoon is known to exist");
/// };
///
/// for episode in webtoon.episodes().await? {
///     println!("episode `{}` for `{}`", episode.number(), webtoon.title());
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
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(837999).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// println!("there are `{}` episodes for `{}`", episodes.count(), webtoon.title());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn count(&self) -> u16 {
        self.count
    }

    /// Wrapper for `Vec::sort()`.
    pub fn sort(&mut self) {
        self.episodes.sort();
    }

    /// Wrapper for `Vec::unstable_sort()`.
    pub fn sort_unstable(&mut self) {
        self.episodes.sort_unstable();
    }

    /// Wrapper for `Vec::unstable_sort_by()`.
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Episode, &Episode) -> Ordering,
    {
        self.episodes.sort_unstable_by(compare);
    }

    /// Creates an immutable slice iterator that does not consume `self`.
    pub fn iter(&self) -> std::slice::Iter<'_, Episode> {
        <&Self as IntoIterator>::into_iter(self)
    }

    /// Gets the episode from the collection.
    ///
    /// If episode is not found in [`Episodes`], then it will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(837999).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    ///
    /// if let Some(episode) = episodes.episode(1) {
    ///     println!("episode `{}` for `{}`", episode.number(), webtoon.title());
    /// }
    ///
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
    fn from(episodes: Vec<Episode>) -> Self {
        Self {
            count: u16::try_from(episodes.len())
                .expect("max episode number should fit within `u16`"),
            episodes,
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
