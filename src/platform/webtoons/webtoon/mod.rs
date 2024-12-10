//! Represents an absraction for a webtoon.

mod dashboard;
pub mod episode;
mod page;

use anyhow::Context;
use core::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "rss")]
pub mod rss;
#[cfg(feature = "rss")]
use rss::Rss;

use self::{
    episode::{posts::Posts, Episode, Episodes},
    page::Page,
};

use super::errors::{ClientError, EpisodeError, PostError, WebtoonError};
use super::meta::{Genre, Scope};
use super::originals::Release;
use super::Type;
use super::{creator::Creator, Client, Language};

// TODO: implement dashboards scraping for other languages

/// Represents a Webtoon from `webtoons.com`.
///
/// This can be thought of as a handle that the methods use to access various parts of the webtoons api for information about the webtoon.
#[derive(Clone)]
pub struct Webtoon {
    pub(super) client: Client,
    pub(super) id: u32,
    pub(super) language: Language,
    // some genre for an original or canvas for canvas webtoons: "fantasy" or "canvas"
    pub(super) scope: Scope,
    /// url slug of the webtoon name: Tower of God -> tower-of-god
    pub(super) slug: Arc<str>,
    /// Cache for data on the Wetboons landing page: title, rating, etc.
    pub(super) page: Arc<Mutex<Option<Page>>>,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Webtoon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Webtoon")
            // omitting `client`
            .field("id", &self.id)
            .field("language", &self.language)
            .field("scope", &self.scope)
            .field("slug", &self.slug)
            .field("page", &self.page)
            .finish()
    }
}

impl Webtoon {
    /// Returns the language of this `Webtoon`.
    pub fn language(&self) -> Language {
        self.language
    }

    /// Returns the id of this `Webtoon`.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the type of this `Webtoon`: Original or Canvas.
    pub fn r#type(&self) -> Type {
        match self.scope {
            Scope::Original(_) => Type::Original,
            Scope::Canvas => Type::Canvas,
        }
    }

    /// Returns the title of this `Webtoon`.
    pub async fn title(&self) -> Result<String, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.title().to_string())
        } else {
            let page = page::scrape(self).await?;

            let title = page.title().to_owned();

            *guard = Some(page);
            drop(guard);

            Ok(title)
        }
    }

    /// Returns a list of [`Creator`] for this `Webtoon`.
    pub async fn creators(&self) -> Result<Vec<Creator>, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.creators().to_vec())
        } else {
            let page = page::scrape(self).await?;

            let creators = page.creators().to_vec();

            *guard = Some(page);
            drop(guard);

            Ok(creators)
        }
    }

    /// Returns a list of [`Genre`] for this `Webtoon`.
    ///
    /// For Originals, the genres are suplumented from the `/genres` page, so you may see more genres than you initially expect.
    pub async fn genres(&self) -> Result<Vec<Genre>, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.genres().to_vec())
        } else {
            let page = page::scrape(self).await?;

            let genres = page.genres().to_vec();

            *guard = Some(page);
            drop(guard);

            Ok(genres)
        }
    }

    /// Returns the summary for this `Webtoon`.
    pub async fn summary(&self) -> Result<String, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.summary().to_owned())
        } else {
            let page = page::scrape(self).await?;

            let summary = page.summary().to_owned();

            *guard = Some(page);
            drop(guard);

            Ok(summary)
        }
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
    pub async fn views(&self) -> Result<u64, EpisodeError> {
        match self.client.get_user_info_for_webtoon(self).await {
            // TODO: Only English dashboards are supported for now.
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                let views = dashboard::episodes::scrape(self)
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

        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.views())
        } else {
            let page = page::scrape(self).await.map_err(|err| match err {
                WebtoonError::ClientError(client_error) => EpisodeError::ClientError(client_error),
                error => EpisodeError::Unexpected(error.into()),
            })?;

            let views = page.views();

            *guard = Some(page);
            drop(guard);

            Ok(views)
        }
    }

    /// Retrieves the total number of subscribers for this `Webtoon`.
    ///
    /// The method determines the subscriber count based on whether the current session belongs to the
    /// creator of the webtoon. If no session is provided or the session doesn't belong to the webtoon creator,
    /// it fetches the subscriber count from the main webtoon page. If the session corresponds to the creator,
    /// it uses the stats dashboard to retrieve a more accurate subscriber count.
    ///
    /// **ONLY ENGLISH DASHBOARDS SUPPORTED**
    /// - If you use with a non-english webtoon, even with a valid session provided that is of the owner of the webtoon
    ///   it will get the public facing page data.  
    ///
    /// ### Behavior
    ///
    /// - **Without Creator Session**: If the current session does not belong to the webtoon creator, or if no session is available,
    ///   this method returns the subscriber count from the webtoon's main page. This count may be less precise, as it can be a rounded number.
    /// - **With Creator Session**: If the session belongs to the creator of the webtoon, this method retrieves subscriber statistics
    ///   directly from the creator's stats dashboard, providing a more accurate and up-to-date count of subscribers.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<u32, WebtoonError>` containing:
    ///
    /// - `Ok(u32)`: The total number of subscribers for the webtoon. If using the creator's session, this value is more precise.
    /// - `Err(WebtoonError)`: An error if the retrieval of subscribers fails, such as due to a client error or a network issue.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// let total_subscribers = webtoon.subscribers().await?;
    /// println!("Total Subscribers: {}", total_subscribers);
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError`: If there is an error with the client, such as a network request failure.
    /// - `WebtoonError::Unexpected`: If an unexpected error occurs during the scraping of the subscribers from either the main page or the stats dashboard.
    ///
    /// ### Notes
    ///
    /// - The accuracy of the subscriber count depends on whether the session matches a webtoon creator. If it does, detailed statistics
    ///   from the creator's dashboard are used for more precision. Otherwise, the subscriber count is retrieved from the webtoon's main page.
    pub async fn subscribers(&self) -> Result<u32, WebtoonError> {
        match self.client.get_user_info_for_webtoon(self).await {
            // TODO: Only english dashboards supported for now
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                let subscribers = dashboard::stats::scrape(self).await?.subscribers;
                return Ok(subscribers);
            }
            // Fallback to public data
            Ok(_) | Err(ClientError::NoSessionProvided) => {}
            Err(err) => return Err(WebtoonError::ClientError(err)),
        }

        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.subscribers())
        } else {
            let page = page::scrape(self).await?;

            let subscribers = page.subscribers();

            *guard = Some(page);
            drop(guard);

            Ok(subscribers)
        }
    }

    /// Returns the rating for this `Webtoon`.
    pub async fn rating(&self) -> Result<f64, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.rating())
        } else {
            let page = page::scrape(self).await?;

            let rating = page.rating();

            *guard = Some(page);
            drop(guard);

            Ok(rating)
        }
    }

    /// Returns the thumbnail url for this `Webtoon`.
    pub async fn thumbnail(&self) -> Result<String, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.thumbnail().to_owned())
        } else {
            let page = page::scrape(self).await?;

            let thumbnail = page.thumbnail().to_owned();

            *guard = Some(page);
            drop(guard);

            Ok(thumbnail)
        }
    }

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
    pub async fn release(&self) -> Result<Option<Vec<Release>>, WebtoonError> {
        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.release().map(|release| release.to_vec()))
        } else {
            let page = page::scrape(self).await?;

            let release = page.release().map(|release| release.to_vec());

            *guard = Some(page);
            drop(guard);

            Ok(release)
        }
    }

    /// Retrieves the banner image URL for this `Webtoon`.
    ///
    /// ### Behavior
    ///
    /// - **Original Webtoons**: If the webtoon is an Original series, this method returns the URL of the banner image, which is typically
    ///   displayed at the top of the webtoon's main page.
    /// - **Canvas Webtoons**: Canvas webtoons do not have banner images, so the method will return `None` if the webtoon is in the Canvas section.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Option<String>, WebtoonError>` containing:
    ///
    /// - `Ok(Some(String))`: A `String` representing the URL of the webtoon's banner image if it is an Original webtoon.
    /// - `Ok(None)`: If the webtoon is a Canvas series, as no banner image is available.
    /// - `Err(WebtoonError)`: An error if the banner image could not be retrieved due to a client or network issue.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// if let Some(banner_url) = webtoon.banner().await? {
    ///     println!("Banner URL: {}", banner_url);
    /// } else {
    ///     println!("This webtoon does not have a banner (Canvas).");
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError`: If there is an issue with the client during the retrieval process.
    /// - `WebtoonError::Unexpected`: If an unexpected error occurs during the scraping of the banner image.
    pub async fn banner(&self) -> Result<Option<String>, WebtoonError> {
        if self.scope == Scope::Canvas {
            return Ok(None);
        }

        let mut guard = self.page.lock().await;

        if let Some(page) = &*guard {
            Ok(page.banner().map(|banner| banner.to_owned()))
        } else {
            let page = page::scrape(self).await?;

            let release = page.banner().map(|release| release.to_owned());

            *guard = Some(page);
            drop(guard);

            Ok(release)
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
        let episodes = match self.client.get_user_info_for_webtoon(self).await {
            // TODO: Only English dashboards are supported for now.
            Ok(user) if user.is_webtoon_creator() && self.language == Language::En => {
                self::dashboard::episodes::scrape(self).await?
            }
            // Fallback to public data
            Ok(_) | Err(ClientError::NoSessionProvided) => {
                page::episodes(self).await.map_err(|err| match err {
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
    pub async fn episode(&self, number: u16) -> Result<Option<Episode>, EpisodeError> {
        let episode = Episode::new(self, number);

        if !episode.exists().await.map_err(|err| match err {
            PostError::ClientError(client_error) => EpisodeError::ClientError(client_error),
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
    /// The feed is optimized to provide a more user-friendly representation of the webtoon’s episodes.
    ///
    /// **CURRENTLY ONLY ENGLISH I SUPPORTED**
    ///
    /// ### Behavior
    ///
    /// - **Episode Data**: The feed includes only episodes that are publicly and freely available, without fast-pass or ad restrictions.
    /// - **Thumbnails**: Instead of including the links to the first two panels of the episode as Webtoons.com provides, this method returns a list of `Episode`s where `Episode::thumbnail` can be used, for example. This design choice aligns more closely with user expectations for an RSS feed, making the implementation more intuitive for feed readers.
    /// - **Optimized Representation**: Some liberties have been taken in formatting the RSS data to better serve typical use cases, making this an opinionated representation of the webtoon’s content.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Rss, WebtoonError>` containing:
    ///
    /// - `Ok(Rss)`: The RSS feed containing episode data.
    /// - `Err(WebtoonError)`: An error if the process fails to retrieve the RSS feed.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// for episode in webtoon.rss().await?.episodes() {
    ///     println!("Episode: {}", episode.title().await?);
    ///     println!("Thumbnail: {}", episode.thumbnail().await?);
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError`: If there is an issue with the client while retrieving the RSS feed.
    /// - `WebtoonError::Unexpected`: If an unexpected error occurs during the process.
    #[cfg(feature = "rss")]
    pub async fn rss(&self) -> Result<Rss, WebtoonError> {
        rss::feed(self).await
    }

    /// Sets the rating of the webtoon for current user session.
    ///
    /// Values are 1-10. Any values outside this range will be clamped to 1 or 10.
    pub async fn rate(&self, rating: u8) -> Result<(), WebtoonError> {
        self.client
            .post_rate_webtoon(self, rating.clamp(1, 10))
            .await?;
        Ok(())
    }

    /// Checks if the current user session is subscribed to the `Webtoon`.
    ///
    /// ### Behavior
    ///
    /// - **Session Required**: This method requires a valid user session to determine if the user has favorited the webtoon.
    ///   - If the session is invalid, it will return an error indicating the problem.
    ///   - If no session is provided, the method returns an error of type `WebtoonError(ClientError::NoSessionProvided)`.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<bool, WebtoonError>` containing:
    ///
    /// - `Ok(true)`: If the user is subscribed (favorited) to the webtoon.
    /// - `Ok(false)`: If the user is not subscribed to the webtoon.
    /// - `Err(WebtoonError::ClientError(ClientError::InvalidSession))`: If the session is invalid or expired.
    /// - `Err(WebtoonError::ClientError(ClientError::NoSessionProvided))`: If no session was provided.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// if webtoon.is_subscribed().await? {
    ///     println!("User is subscribed to this webtoon.");
    /// } else {
    ///     println!("User is not subscribed.");
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError(ClientError::InvalidSession)`: If the session is invalid or expired.
    /// - `WebtoonError::ClientError(ClientError::NoSessionProvided)`: If no user session was provided.
    /// - `WebtoonError::Unexpected`: If an unexpected issue occurs during the process of retrieving user information.
    pub async fn is_subscribed(&self) -> Result<bool, WebtoonError> {
        Ok(self.client.get_user_info_for_webtoon(self).await?.favorite)
    }

    /// Subscribes the current user to the `Webtoon`, if not already subscribed.
    ///
    /// ### Behavior
    ///
    /// - **Creator Check**: The method checks if the user is the creator of the webtoon.
    ///   - If the user is the creator, the method does nothing and immediately returns `Ok(())`, as creators cannot subscribe to their own webtoons.
    ///
    /// - **Subscription Status Check**:
    ///   - If the user is already subscribed, the method returns `Ok(())` without taking any further action.
    ///
    /// - **Subscribing**:
    ///   - After a successful API call, the method returns `Ok(())`.
    ///
    /// ### Returns
    ///
    /// This method returns a `Result<(), WebtoonError>`:
    ///
    /// - `Ok(())`: If the user is already subscribed, or the subscription is successful.
    /// - `Err(WebtoonError::ClientError(ClientError::InvalidSession))`: If the session is invalid or expired.
    /// - `Err(WebtoonError::ClientError(ClientError::NoSessionProvided))`: If no session is provided.
    /// - `Err(WebtoonError::Unexpected)`: If an unexpected issue occurs during the subscription process.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// webtoon.subscribe().await?;
    /// println!("Subscription successful.");
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError(ClientError::InvalidSession)`: If the session is invalid or expired.
    /// - `WebtoonError::ClientError(ClientError::NoSessionProvided)`: If no user session was provided.
    /// - `WebtoonError::Unexpected`: If an unexpected issue occurs during the process of subscribing.
    pub async fn subscribe(&self) -> Result<(), WebtoonError> {
        let user = self.client.get_user_info_for_webtoon(self).await?;

        // Can't sub to own webtoon
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
    /// ### Behavior
    ///
    /// - **Creator Check**: The method checks if the user is the creator of the webtoon.
    ///   - If the user is the creator, the method does nothing and returns `Ok(())`, since creators cannot unsubscribe from their own webtoons.
    ///
    /// - **Subscription Status Check**:
    ///   - If the user is not subscribed, the method returns `Ok(())` without taking any further action.
    ///
    /// - **Unsubscribing**:
    ///   - After a successful API call, the method returns `Ok(())`.
    ///
    /// ### Returns
    ///
    /// This method returns a `Result<(), WebtoonError>`:
    ///
    /// - `Ok(())`: If the user is not subscribed, or the unsubscription is successful.
    /// - `Err(WebtoonError::ClientError(ClientError::InvalidSession))`: If the session is invalid or expired.
    /// - `Err(WebtoonError::ClientError(ClientError::NoSessionProvided))`: If no session is provided.
    /// - `Err(WebtoonError::Unexpected)`: If an unexpected issue occurs during the unsubscription process.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// webtoon.unsubscribe().await?;
    /// println!("Unsubscription successful.");
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Errors
    ///
    /// - `WebtoonError::ClientError(ClientError::InvalidSession)`: If the session is invalid or expired.
    /// - `WebtoonError::ClientError(ClientError::NoSessionProvided)`: If no user session was provided.
    /// - `WebtoonError::Unexpected`: If an unexpected issue occurs during the unsubscription process.
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

    /// Clears the cached metadata for the current `Webtoon`, forcing future requests to retrieve fresh data from the network.
    ///
    /// ### Behavior
    ///
    /// - **Cache Eviction**:
    ///   - This method clears the cached webtoon metadata (such as genre, title, and other page information) that has been stored for performance reasons.
    ///   - After calling this method, subsequent calls that rely on this metadata will trigger a network request to re-fetch the data.
    ///
    /// ### Use Case
    ///
    /// - Use this method if you suspect the cached data is outdated or if you want to ensure that future data retrieval reflects the latest updates from the webtoon's page.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// webtoon.evict_cache().await;
    /// println!("Cache cleared. Future requests will fetch fresh data.");
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Notes
    ///
    /// - There are no errors returned from this function, as it only resets the cache.
    /// - Cache eviction is useful if the webtoon metadata has changed or if up-to-date information is needed for further operations.
    pub async fn evict_cache(&self) {
        let mut page = self.page.lock().await;
        *page = None;
    }
}

// Internal use
impl Webtoon {
    pub(super) async fn new_with_client(
        id: u32,
        r#type: Type,
        client: &Client,
    ) -> Result<Self, anyhow::Error> {
        let url = format!(
            "https://www.webtoons.com/*/{}/*/list?title_no={id}",
            match r#type {
                Type::Original => "*",
                Type::Canvas => "canvas",
            }
        );

        let response = client.http.get(&url).send().await?;

        // Webtoon doesn't exist
        if response.status() == 404 {
            anyhow::bail!("Webtoon should always exist when using `new_with_client` which is designed for internal use only.");
        }

        let mut segments = response
            .url()
            .path_segments()
            .ok_or(WebtoonError::InvalidUrl(
                "Webtoon url should have segments separated by `/`; this url did not.",
            ))?;

        let segment = segments
            .next()
            .ok_or(WebtoonError::InvalidUrl(
                "Webtoon URL was found to have segments, but for some reason failed to extract that first segment, which should be a language code: e.g `en`",
            ))?;

        let language = Language::from_str(segment)
            .context("Failed to parse return URL segment into `Language` enum")?;

        let segment = segments.next().ok_or(
                WebtoonError::InvalidUrl("Url was found to have segments, but didn't have a second segment, representing the scope of the webtoon.")
            )?;

        let scope = Scope::from_str(segment) //
            .context("Failed to parse URL scope path to a `Scope`")?;

        let slug = segments
            .next()
            .ok_or( WebtoonError::InvalidUrl( "Url was found to have segments, but didn't have a third segment, representing the slug name of the Webtoon."))?
            .to_string();

        let webtoon = Webtoon {
            client: client.clone(),
            id,
            language,
            scope,
            slug: Arc::from(slug),
            page: Arc::new(Mutex::new(None)),
        };

        Ok(webtoon)
    }

    pub(super) fn from_url_with_client(url: &str, client: &Client) -> Result<Self, anyhow::Error> {
        let url = url::Url::parse(url).map_err(|err| WebtoonError::Unexpected(err.into()))?;

        let mut segments = url
            .path_segments()
            .context("webtoon url should have segments")?;

        let id = url
            .query()
            .context("webtoon url should have a `title_no` query")?
            .split('=')
            .nth(1)
            .context("`title_no` should always have a `=`")?
            .parse::<u32>()
            .context("`title_no` query parameter wasn't able to parse into a u32")?;

        let language = Language::from_str(
            segments
                .next()
                .context("webtoon url should have a language segment as its first")?,
        )?;

        let scope = Scope::from_str(
            segments
                .next()
                .context("webtoon url should have a scope segment as its second")?,
        )
        .with_context(|| format!("id: `{id}` had an unknown genre slug"))?;

        let slug = segments
            .next()
            .context("webtoon url should have a slug segment as its third")?
            .to_string();

        let webtoon = Self {
            client: client.clone(),
            language,
            scope,
            slug: Arc::from(slug),
            id,
            page: Arc::new(Mutex::new(None)),
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
