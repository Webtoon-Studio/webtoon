//! Represents an abstraction for a webtoon.

pub mod episode;

use anyhow::Context;
use core::fmt;

use self::episode::{posts::Posts, Episode, Episodes};

use super::client::info::{self, Info};
use super::errors::{ClientError, EpisodeError, PostError, WebtoonError};
use super::meta::{Genre, Release};
use super::Type;
use super::{creator::Creator, Client};

/// Represents a Webtoon from `comic.naver.com`.
///
/// This can be thought of as a handle that the methods use to access various parts of the naver api for information about the webtoon.
#[derive(Clone)]
pub struct Webtoon {
    pub(super) client: Client,
    pub(super) id: u64,
    pub(super) r#type: Type,
    pub(super) creators: Vec<Creator>,
    pub(super) title: String,
    pub(super) genres: Vec<Genre>,
    pub(super) favorites: u32,
    pub(super) summary: String,
    pub(super) thumbnail: String,
    pub(super) weekdays: Vec<Release>,
    pub(super) is_completed: bool,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Webtoon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Webtoon")
            // Omitting `client`
            .field("id", &self.id)
            .field("type", &self.r#type)
            .field("creators", &self.creators)
            .field("title", &self.title)
            .field("genres", &self.genres)
            .field("favorites", &self.favorites)
            .field("summary", &self.summary)
            .field("thumbnail", &self.thumbnail)
            .field("weekdays", &self.weekdays)
            .field("is_completed", &self.is_completed)
            .finish()
    }
}

impl Webtoon {
    /// Returns the id of this `Webtoon`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the type of this `Webtoon`: Original or Canvas.
    pub fn r#type(&self) -> Type {
        self.r#type
    }

    /// Returns the title of this `Webtoon`.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns a list of [`Creator`] for this `Webtoon`.
    pub fn creators(&self) -> &[Creator] {
        &self.creators
    }

    /// Returns a list of [`Genre`] for this `Webtoon`.
    ///
    /// For Originals, the genres are supplemented from the `/genres` page, so you may see more genres than you initially expect.
    pub fn genres(&self) -> &[Genre] {
        &self.genres
    }

    /// Returns the summary for this `Webtoon`.
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Retrieves the total number of views for this `Webtoon`.
    ///
    /// The method determines the total views based on whether the current session belongs to the creator
    /// of the webtoon. If no session is provided or the session doesn't belong to the webtoon creator,
    /// it fetches the views directly from the main webtoon page. If the session corresponds to the creator,
    /// it uses episode data to sum up views for each episode, providing a more precise count.
    ///
    /// **ONLY ENGLISH DASHBOARD SUPPORTED**
    /// - Even if valid session is provided for the webtoon creator, only the public data on webtoon page will be gotten.
    ///
    /// ### Behavior
    ///
    /// - **Without Creator Session**: If the current session does not belong to the webtoon creator, or if no session is available, this method returns the view count from the webtoon's main page. This value may be rounded (e.g., `3,800,000`).
    /// - **With Creator Session**: If the session belongs to the creator of the webtoon, this method fetches more detailed episode-by-episode view counts and sums them to return a more accurate total view count (e.g., `3,804,237` instead of `3,800,000`).
    ///
    /// ### Returns
    ///
    /// Returns a `Result<u64, EpisodeError>` containing:
    ///
    /// - `Ok(u64)`: The total number of views for the webtoon. If using the creator's session, this value is more precise.
    /// - `Err(EpisodeError)`: An error if the webtoon view retrieval fails, such as due to a client error or a network issue.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(95, Type::Original).await? {
    /// let total_views = webtoon.views().await?;
    /// println!("Total Views: {}", total_views);
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `EpisodeError::ClientError`: If there is an error with the client, such as a network request failure.
    /// - `EpisodeError::Unexpected`: If an unexpected error occurs during the scraping of the episode views.
    ///
    /// ### Notes
    ///
    /// - The accuracy of the view count depends on whether the session matches a webtoon creator. If it does, detailed episode
    ///   data is used for more precision. Otherwise, the views from the main page are returned.
    async fn views(&self) -> Result<u64, EpisodeError> {
        todo!()
    }

    pub fn favorites(&self) -> u32 {
        self.favorites
    }

    /// Returns the rating for this `Webtoon`.
    async fn rating(&self) -> Result<f64, WebtoonError> {
        todo!()
    }

    /// Returns the thumbnail url for this `Webtoon`.
    pub fn thumbnail(&self) -> &str {
        &self.thumbnail
    }

    // TODO
    /// Retrieves the release schedule for this `Webtoon`.
    ///
    /// ### Behavior
    ///
    /// - **Original Webtoons**: If the webtoon is an Original series, this method fetches the release schedule and
    ///   returns it as a `Vec<Release>`. The release schedule contains information about upcoming or regular episode drops.
    /// - **Canvas Webtoons**: If the webtoon is part of the Canvas section, there is no official release schedule, and this
    ///   method will return `None`.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Option<Vec<Release>>, WebtoonError>` containing:
    ///
    /// - `Ok(Some(Vec<Release>))`: A vector of `Release` objects that detail the webtoon's release schedule if the webtoon is an Original.
    /// - `Ok(None)`: If the webtoon is a Canvas series, the release schedule is not available.
    /// - `Err(WebtoonError)`: An error if scraping the release schedule fails due to a client or network issue.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// if let Some(release_schedule) = webtoon.release().await? {
    ///     for release in release_schedule {
    ///         println!("Upcoming release: {release:?}");
    ///     }
    /// } else {
    ///     println!("This webtoon does not have a release schedule (Canvas).");
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError`: If there is an issue with the client during the retrieval process.
    /// - `WebtoonError::Unexpected`: If an unexpected error occurs during the scraping of the release schedule.
    pub fn weekdays(&self) -> Option<&[Release]> {
        match self.r#type {
            Type::Original => Some(&self.weekdays),
            Type::Canvas => None,
        }
    }

    /// Retrieves all episodes for the current `Webtoon`. The method's behavior depends on whether the user is the creator of the webtoon
    /// or a regular viewer (i.e., a session is provided or not, and if the session user is a creator for the webtoon).
    ///
    /// **ONLY ENGLISH DASHBOARD SUPPORTED**
    /// - Even if valid session is provided for the webtoon creator, only the public data will be gotten.
    ///
    /// ### Behavior
    ///
    /// - **For Creators**: If the session belongs to the creator of the webtoon, the method will scrape the episode dashboard. This includes:
    ///   - All episodes, including drafts.
    ///   - Episodes behind fast-pass or ad walls.
    ///   - Access to additional episode metadata, such as view counts (`Episode::views()` will return `Some(u32)` for episodes retrieved via the dashboard).
    ///   - Fully accurate publication times (`Episode::published()` will return the exact timestamp of the episode's release).
    ///
    /// - **For Regular Users**: If the session is not provided or the user is not the creator of the webtoon, the method will scrape the publicly available episodes:
    ///   - Only episodes that are publicly visible on the webtoon's main page will be retrieved.
    ///   - Episodes behind fast-pass or ad walls will not be included.
    ///   - View counts (`Episode::views()`) will return `None` for episodes retrieved from the main page as the information is unavailable.
    ///   - The publication time (`Episode::published()`) will return `Some(i64)` but the time will always be set to `2:00 AM` on the episode's published date.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Episodes, EpisodeError>` containing:
    ///
    /// - `Ok(Episodes)`: An `Episodes` struct.
    /// - `Err(EpisodeError)`: An error if the episode scraping process fails due to a client error or unexpected issue.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// let episodes = webtoon.episodes().await?;
    /// println!("Total episodes: {}", episodes.count());
    ///
    /// for episode in episodes {
    ///     println!("Episode title: {}", episode.title().await?);
    ///     if let Some(views) = episode.views() {
    ///         println!("Views: {}", views);
    ///     } else {
    ///         println!("Views are unavailable for this episode.");
    ///     }
    ///
    ///     if let Some(published) = episode.published() {
    ///         println!("Published at: {}", published);
    ///     } else {
    ///         panic!("Publish date should always be available with `episodes`.");
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
    pub async fn episodes(&self) -> Result<Episodes, EpisodeError> {
        let mut episodes = Vec::new();

        let response = self.client.webtoon_episodes_at_page(self, 1).await?;

        let pages = response.page_info.total_pages;

        // Add paid episodes first
        for episode in response.charge_folder_article_list {
            episodes.push(Episode::from((self, episode)));
        }

        for episode in response.article_list {
            episodes.push(Episode::from((self, episode)));
        }

        for page in 2..=pages {
            let response = self.client.webtoon_episodes_at_page(self, page).await?;

            for episode in response.article_list {
                episodes.push(Episode::from((self, episode)));
            }
        }

        Ok(Episodes {
            count: episodes.len() as u16,
            episodes,
        })
    }

    /// Constructs an `Episode` if it exists.
    ///
    /// However, there are important caveats to be aware of when using this method instead of `episodes`.
    ///
    /// ### Caveats
    ///
    /// - **No View or Publish Data**:
    ///   - `Episode::views()` will always return `None`. This method does not provide view counts; you must use `episodes()` to retrieve views.
    ///   - `Episode::published()` will always return `None`. To get publication dates, you must use `episodes()`. This is a limitation as currently there is no known way to get the published date with just the episode value alone.
    ///
    /// - **Episode Existence vs. Public Display**:
    ///   - This method includes episodes that are unpublished, behind ads or fast-pass, or even "deleted" (i.e., episodes that no longer appear on the main page but are still accessible through their episode number).
    ///   - It does not rely solely on public episodes, meaning it will count and retrieve episodes that a regular user would not normally see without having access to a creator's dashboard or matching creator-webtoon session.
    ///   - The numbering (`#NUMBER`) of episodes retrieved by this method may differ from public episode lists due to the inclusion of hidden or removed episodes. You can see the matching episode with the `episode_no=` query in the URL.
    ///
    /// ### Use Cases
    ///
    /// - Accessing hidden episodes, such as those unpublished, behind fast-pass, ad-walled, or deleted, without requiring a matching creator session.
    /// - Useful for situations where the complete set of episodes is necessary, including drafts or episodes not currently visible to the public.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Option<Episode>, EpisodeError>` containing:
    ///
    /// - `Ok(Some(Episode))`: If the episode exists (including hidden or deleted ones).
    /// - `Ok(None)`: If the episode does not exist.
    /// - `Err(EpisodeError)`: If there is an error during the episode retrieval process.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// if let Some(episode) = webtoon.episode(42).await? {
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
    // pub async fn episode(&self, number: u16) -> Result<Option<Episode>, EpisodeError> {
    // let episode = Episode::new(self, number);

    // if !episode.exists().await.map_err(|err| match err {
    //     PostError::ClientError(client_error) => EpisodeError::ClientError(client_error),
    //     error => EpisodeError::Unexpected(error.into()),
    // })? {
    //     return Ok(None);
    // }

    // Ok(Some(episode))
    //     todo!()
    // }

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
    /// Returns a `Result<u32, EpisodeError>` containing:
    ///
    /// - `Ok(u32)`: The total number of likes across all episodes.
    /// - `Err(EpisodeError)`: If there is an issue during the retrieval process for any episode.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
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
        todo!()
    }

    /// Retrieves all posts(top level comments) for every episode of the current `Webtoon`.
    ///
    /// This method can return more posts than what is publicly available on the main webtoon page, as it includes certain deleted posts as well as those visible to all users.
    ///
    /// ### Behavior
    ///
    /// - The method retrieves all posts across every episode, including:
    ///   - **Publicly visible posts**: Comments that any user can see on the webtoon page.
    ///   - **Deleted posts with replies**: Posts that have been marked as deleted but still display the message "This comment has been deleted" because they have replies.
    ///   - **Excluded posts**: Deleted posts without any replies are not included in the results.
    ///
    /// ### Clarification
    ///
    /// - The method may return more posts than expected since it counts comments that are deleted but still display a "This comment has been deleted" message (due to having replies).
    /// - **Hidden posts**: Posts from episodes behind fast-pass, ads, or even deleted/unpublished episodes will also be included, provided that they have replies and match the pattern mentioned above.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Posts, PostError>` containing:
    ///
    /// - `Ok(Posts)`: A `Posts` struct containing all posts retrieved.
    /// - `Err(PostError)`: An error if the process fails while retrieving posts.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// let posts = webtoon.posts().await?;
    /// for post in posts {
    ///     println!("Post: {}", post.body().contents());
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
    pub async fn posts(&self) -> Result<(), PostError> {
        // let mut posts = Vec::new();

        // for number in 1.. {
        //     if let Some(episode) = self.episode(number).await.map_err(|err| match err {
        //         EpisodeError::ClientError(client_error) => PostError::ClientError(client_error),
        //         error => PostError::Unexpected(error.into()),
        //     })? {
        //         posts.extend_from_slice(episode.posts().await?.as_slice());
        //     } else {
        //         break;
        //     }
        // }

        // Ok(posts.into())
        todo!()
    }

    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
}

impl From<(Info, &Client)> for Webtoon {
    fn from((info, client): (Info, &Client)) -> Self {
        Self {
            client: client.clone(),
            id: info.id,
            r#type: info.r#type,
            creators: info
                .creators
                .into_iter()
                .map(|creator| Creator::from((creator, client)))
                .collect(),
            title: info.title,
            genres: info.data.genres,
            favorites: info.favorites,
            summary: info.summary,
            thumbnail: info.thumbnail,
            weekdays: info.weekdays,
            is_completed: info.is_completed,
        }
    }
}
