//! Represents an abstraction for a Webtoon.

pub mod episode;

use super::client::episodes::{Root, Sort};
use super::errors::{EpisodeError, PostError, WebtoonError};
use super::meta::Genre;
use super::{Client, creator::Creator};
use super::{Type, meta::Weekday};
use core::fmt;
use episode::{Episode, Episodes, posts::Posts};
use std::sync::Arc;

/// Represents a Webtoon from `comic.naver.com`.
///
/// This can be thought of as a handle that the methods use to access various parts of the Naver's API for information about the Webtoon.
#[derive(Clone)]
pub struct Webtoon {
    pub(super) client: Client,
    pub(super) inner: Arc<WebtoonInner>,
}

pub(super) struct WebtoonInner {
    pub id: u32,
    pub r#type: Type,
    pub title: String,
    pub thumbnail: String,
    pub summary: String,
    pub is_new: bool,
    pub on_hiatus: bool,
    pub is_completed: bool,
    pub favorites: u32,
    pub schedule: Vec<Weekday>,
    pub genres: Vec<Genre>,

    pub creators: Vec<Creator>,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Webtoon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Webtoon")
            // omitting `client`
            .field("id", &self.inner.id)
            .field("type", &self.inner.r#type)
            .field("title", &self.inner.title)
            .field("thumbnail", &self.inner.thumbnail)
            .field("summary", &self.inner.summary)
            .field("is_new", &self.inner.is_new)
            .field("on_hiatus", &self.inner.on_hiatus)
            .field("is_completed", &self.inner.is_completed)
            .field("favorites", &self.inner.favorites)
            .field("schedule", &self.inner.schedule)
            .field("genres", &self.inner.genres)
            .field("creators", &self.inner.creators)
            .finish()
    }
}

impl Webtoon {
    /// Returns the id of this `Webtoon`.
    pub fn id(&self) -> u32 {
        self.inner.id
    }

    /// Returns the type of this `Webtoon`: Featured, BestChallenge or Challenge.
    pub fn r#type(&self) -> Type {
        self.inner.r#type
    }

    /// Returns if Webtoon is a featured type.
    pub fn is_featured(&self) -> bool {
        self.r#type() == Type::Featured
    }

    /// Returns if Webtoon is a best challenge type.
    pub fn is_best_challenge(&self) -> bool {
        self.r#type() == Type::BestChallenge
    }

    /// Returns if Webtoon is a challenge type.
    pub fn is_challenge(&self) -> bool {
        self.r#type() == Type::Challenge
    }

    /// Returns the title of this `Webtoon`.
    pub fn title(&self) -> &str {
        &self.inner.title
    }

    /// Returns a list of [`Creator`] for this `Webtoon`.
    pub fn creators(&self) -> &[Creator] {
        &self.inner.creators
    }

    /// Returns a list of [`Genre`] for this `Webtoon`.
    pub fn genres(&self) -> &[Genre] {
        &self.inner.genres
    }

    /// Returns the summary for this `Webtoon`.
    pub fn summary(&self) -> &str {
        &self.inner.summary
    }

    /// Retrieves the total number of favorites for this `Webtoon`.
    pub fn favorites(&self) -> u32 {
        self.inner.favorites
    }

    /// Returns the rating for this `Webtoon`.
    pub async fn rating(&self) -> Result<f64, WebtoonError> {
        let episodes = self
            .episodes(Sort::Asc)
            .await
            .map_err(|err| WebtoonError::Unexpected(err.into()))?;

        let count = episodes.count();

        let mut rating = 0.0;
        for episode in &episodes {
            rating += episode.rating().await.map_err(|err| match err {
                EpisodeError::ClientError(client_error) => WebtoonError::ClientError(client_error),
                EpisodeError::Unexpected(error) => WebtoonError::Unexpected(error),
                _ => unreachable!(),
            })?;
        }

        Ok(rating / count as f64)
    }

    /// Returns the thumbnail URL for this `Webtoon`.
    pub fn thumbnail(&self) -> &str {
        &self.inner.thumbnail
    }

    /// Retrieves the release schedule for this `Webtoon`.
    ///
    /// ### Behavior
    ///
    /// - **Challenge Webtoon**: There is no official release schedule, and this method will return `None`.
    pub fn schedule(&self) -> Option<&[Weekday]> {
        if self.is_challenge() {
            return None;
        }

        Some(&self.inner.schedule)
    }

    /// Returns if `Webtoon` is completed.
    ///
    /// **NOTE**: This has been known to be incorrect, trust with extreme caution!
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed
    }

    /// Returns if `Webtoon` is new.
    pub fn is_new(&self) -> bool {
        self.inner.is_new
    }

    /// Returns if `Webtoon` is on a hiatus.
    pub fn is_on_hiatus(&self) -> bool {
        self.inner.on_hiatus
    }

    /// Retrieves all episodes for the current `Webtoon`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// let episodes = webtoon.episodes(Sort::Asc).await?;
    /// println!("Total episodes: {}", episodes.count());
    ///
    /// for episode in &episodes {
    ///     println!("Episode title: {}", episode.title().await?);
    ///
    ///     // Episodes behind cookies do not have a published date.
    ///     if let Some(published) = episode.published().await? {
    ///         println!("Published at: {}", published);
    ///     }
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `EpisodeError::ClientError`: If there is an issue with the client during the retrieval process.
    /// - `EpisodeError::Unexpected`: If an unexpected error occurs during the scraping of episode data.
    pub async fn episodes(&self, sort: Sort) -> Result<Episodes, EpisodeError> {
        let mut episodes = Vec::new();

        let response = self.client.get_episodes_json(self, 1, sort).await?;

        let txt = response.text().await?;

        let json: Root = serde_json::from_str(&txt) //
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let pages = json.page_info.total_pages;

        // Add first episodes
        for article in json.article_list {
            episodes.push(Episode::from((self.clone(), article)));
        }

        // TODO: Unsure what happens if there is more than 20 charged episodes.
        // Add paid episodes
        for article in json.charge_folder_article_list {
            episodes.push(Episode::from((self.clone(), article)));
        }

        for page in 2..=pages {
            let response = self.client.get_episodes_json(self, page, sort).await?;

            let txt = response.text().await?;

            let json: Root = serde_json::from_str(&txt) //
                .map_err(|err| EpisodeError::Unexpected(err.into()))?;

            for article in json.article_list {
                episodes.push(Episode::from((self.clone(), article)));
            }
        }

        match sort {
            Sort::Asc => episodes.sort_unstable(),
            Sort::Desc => episodes.sort_unstable_by(|a, b| b.cmp(a)),
        }

        let episodes = Episodes {
            count: episodes.len() as u16,
            episodes,
        };

        Ok(episodes)
    }

    /// Constructs an `Episode` if it exists.
    ///
    /// However, there are important caveats to be aware of when using this method instead of `episodes`.
    ///
    /// ### Caveats
    ///
    /// - **Episode Existence vs. Public Display**:
    ///   - This method includes episodes that are unpublished, behind ads or fast-pass, or even "deleted" (i.e., episodes that no longer appear on the main page but are still accessible through their episode number).
    ///   - It does not rely solely on public episodes, meaning it will count and retrieve episodes that a regular user would not normally see without having access to a creator's dashboard.
    ///   - The numbering of episodes retrieved by this method may differ from public episode lists due to the inclusion of hidden or removed episodes. You can see the matching episode with the `no=` query in the URL.
    ///
    /// ### Returns
    ///
    /// Will return a `Result<Option<Episode>, EpisodeError>` containing:
    ///
    /// - `Ok(Some(Episode))`: If the episode exists (including hidden or deleted ones).
    /// - `Ok(None)`: If the episode does not exist.
    /// - `Err(EpisodeError)`: If there is an error during the episode retrieval process.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{ Client, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     println!("Episode title: {}", episode.title().await?);
    /// } else {
    ///     println!("Episode 42 does not exist.");
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `EpisodeError::ClientError`: If there is an issue with the client during the retrieval process.
    /// - `EpisodeError::Unexpected`: If an unexpected error occurs during the scraping or episode validation process.
    pub async fn episode(&self, number: u16) -> Result<Option<Episode>, EpisodeError> {
        let episode = Episode::new(self, number);

        if !episode.exists().await.map_err(|err| match err {
            EpisodeError::ClientError(client_error) => EpisodeError::ClientError(client_error),
            error => EpisodeError::Unexpected(error.into()),
        })? {
            return Ok(None);
        }

        Ok(Some(episode))
    }

    /// Retrieves the total number of likes for all episodes of the current `Webtoon`.
    ///
    /// This including those behind ads, fast-pass, or even deleted episodes. This can lead to a discrepancy between the publicly displayed episodes and the actual total likes, as it accounts for episodes that are normally hidden or restricted from public view.
    ///
    /// ### Behavior
    ///
    /// - This method sums the likes across all episodes, regardless of whether the episodes are:
    ///   - **Publicly available**: It includes public episodes that any user can see.
    ///   - **Hidden**: Episodes behind fast-pass or ad walls.
    ///   - **Deleted**: Episodes that no longer appear on the public main page but still exist in the system.
    ///   - **Unpublished**: Drafts or episodes not yet publicly released.
    ///
    /// ### Clarification
    ///
    /// - **Like Discrepancy**: The total likes calculated here may not match the likes visible to a regular user on the public webtoon page due to the inclusion of hidden, unpublished, and deleted episodes.
    ///
    /// ### Returns
    ///
    /// Will return a `Result<u32, EpisodeError>` containing:
    ///
    /// - `Ok(u32)`: The total number of likes across all episodes.
    /// - `Err(EpisodeError)`: If there is an issue during the retrieval process for any episode.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{ Client, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// let total_likes = webtoon.likes().await?;
    /// println!("Total likes for the webtoon: {}", total_likes);
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `EpisodeError::ClientError`: If there is an issue with the client during episode retrieval.
    /// - `EpisodeError::Unexpected`: If an unexpected error occurs during the process.
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

    /// Retrieves all posts(top level comments) for every episode of the current `Webtoon`.
    ///
    /// ### Returns
    ///
    /// Will returns a `Result<Posts, PostError>` containing:
    ///
    /// - `Ok(Posts)`: A `Posts` struct containing all posts retrieved.
    /// - `Err(PostError)`: An error if the process fails while retrieving posts.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// let posts = webtoon.posts().await?;
    /// for post in posts {
    ///     println!("Post: {}", post.body());
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `PostError::ClientError`: If there is an issue with the client during episode or post retrieval.
    /// - `PostError::Unexpected`: If an unexpected error occurs during the process.
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
}

// Internal use
impl Webtoon {
    pub(super) async fn new_with_client(id: u32, client: &Client) -> Result<Self, anyhow::Error> {
        Ok(client
            .webtoon(id)
            .await?
            .expect("Webtoon should always exist with given `id` as this is internal use only"))
    }
}
