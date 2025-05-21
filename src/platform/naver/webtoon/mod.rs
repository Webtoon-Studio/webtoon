//! Represents an abstraction for a Webtoon on `comic.naver.com`.

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
/// This type is not constructed directly, instead it is gotten through a [`Client`] via [`Client::webtoon()`] or [`Client::webtoon_from_url()`].
///
/// This abstracts over all the sections a Webtoon may be in, such the featured or challenge sections. Relevant capabilities
/// are exposed, with methods taking missing features that may not exists across all sections.
///
/// Read the method documentation for any notes on section interactions to take into account.
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
    ///
    /// Most often, this is just the given id passed when constructing a [`Webtoon`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(832703).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(832703, webtoon.id());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn id(&self) -> u32 {
        self.inner.id
    }

    /// Returns the title of this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838432).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!("우렉 마지노", webtoon.title());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn title(&self) -> &str {
        &self.inner.title
    }

    /// Returns the type of this [`Webtoon`]: [`Featured`](variant@Type::Featured), [`BestChallenge`](variant@Type::BestChallenge) or [`Challenge`](variant@Type::Challenge).
    ///
    /// # Variants
    ///
    /// - `Type::Featured`: Webtoon's in the `comic.naver.com/webtoon` section.
    /// - `Type::BestChallenge`: Webtoon's in the `comic.naver.com/bestChallenge` section.
    /// - `Type::Challenge`: Webtoon's in the `comic.naver.com/challenge` section.
    ///
    /// # Alternatives
    ///
    /// For simple `bool` checks, check out [`is_featured()`](Webtoon::is_featured()), [`is_best_challenge()`](Webtoon::is_best_challenge()), or [`is_challenge()`](Webtoon::is_challenge()).
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838432).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(Type::Featured, webtoon.r#type());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn r#type(&self) -> Type {
        self.inner.r#type
    }

    /// Returns if Webtoon is a [`Featured`](variant@Type::Featured) type.
    ///
    /// Is `true` if the Webtoon is in the `comic.naver.com/webtoon` section, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838432).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_featured());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_featured(&self) -> bool {
        self.r#type() == Type::Featured
    }

    /// Returns if Webtoon is a [`BestChallenge`](variant@Type::BestChallenge) type.
    ///
    /// Is `true` if the Webtoon is in the `comic.naver.com/bestChallenge` section, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(816103).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_best_challenge());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_best_challenge(&self) -> bool {
        self.r#type() == Type::BestChallenge
    }

    /// Returns if Webtoon is a [`Challenge`](variant@Type::Challenge) type.
    ///
    /// Is `true` if the Webtoon is in the `comic.naver.com/challenge` section, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(758207).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_challenge());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_challenge(&self) -> bool {
        self.r#type() == Type::Challenge
    }

    /// Returns a slice of [`Creator`] for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(828715).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let creators = webtoon.creators();
    ///
    /// assert!(creators.len() == 3);
    /// assert_eq!("JP", creators[0].username());
    /// assert_eq!("박진환", creators[1].username());
    /// assert_eq!("장영훈", creators[2].username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn creators(&self) -> &[Creator] {
        &self.inner.creators
    }

    /// Returns a slice of [`Genre`] for this `Webtoon`.
    ///
    /// Specifically, these are only the traditional genres, and not other tags.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Genre, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(822657).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let genres = webtoon.genres();
    ///
    /// assert!(genres.len() == 1);
    /// assert_eq!(Genre::Historical, genres[0]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn genres(&self) -> &[Genre] {
        &self.inner.genres
    }

    /// Returns the summary for this `Webtoon`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(747269).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let expected = "'이건 내가 아는 그 전개다'\n한순간에 세계가 멸망하고, 새로운 세상이 펼쳐졌다.\n오직 나만이 완주했던 소설 세계에서 평범했던 독자의 새로운 삶이 시작된다.";
    ///
    /// assert_eq!(expected, webtoon.summary());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn summary(&self) -> &str {
        &self.inner.summary
    }

    /// Retrieves the total number of favorites for this `Webtoon`.
    ///
    /// Specifically, this is the number for `관심`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(820097).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("favorites: {}", webtoon.favorites());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn favorites(&self) -> u32 {
        self.inner.favorites
    }

    /// Returns the rating for this `Webtoon`.
    ///
    /// # Discrepancies
    ///
    /// This is the mean of all free and paid episodes. Elsewhere a rating might be shown on the platform, but that is
    /// the mean for only the free episodes. If there is a discrepancy, it is most likely due to this.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(822862).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("rating: {}", webtoon.rating().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rating(&self) -> Result<f64, WebtoonError> {
        let episodes = self
            .episodes()
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
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(829875).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("thumbnail url: {}", webtoon.thumbnail());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn thumbnail(&self) -> &str {
        &self.inner.thumbnail
    }

    /// Retrieves the release schedule for this `Webtoon`.
    ///
    /// # Caveats
    ///
    /// For [`Challenge`](variant@Type::Challenge) Webtoon's there is no official release schedule, and this method will always return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, meta::Weekday, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(694721).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!(Some(&[Weekday::Monday][..]), webtoon.schedule());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn schedule(&self) -> Option<&[Weekday]> {
        if self.is_challenge() {
            return None;
        }

        Some(&self.inner.schedule)
    }

    /// Returns if `Webtoon` is completed.
    ///
    /// <div class="warning">
    ///
    /// **This has been known to be incorrect, trust with extreme caution!**
    ///
    /// </div>
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(795658).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(webtoon.is_completed());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed
    }

    /// Returns if the `Webtoon` is considered new.
    ///
    /// This corresponds to the `신작` badge. Generally, the platform seems to consider any Webtoon who's first episode
    /// was published within the last 30-days as new.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(747271).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert!(!webtoon.is_new());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_new(&self) -> bool {
        self.inner.is_new
    }

    /// Returns if the `Webtoon` is on a hiatus.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(829195).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("{} is {}on hiatus",webtoon.title(), if webtoon.is_on_hiatus() { "" } else {"not "});
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_on_hiatus(&self) -> bool {
        self.inner.on_hiatus
    }

    /// Retrieves all [`Episodes`] for the current `Webtoon`.
    ///
    /// `Episodes` is returned sorted in ascending order: 1, 2, 3, 4, etc. Only episodes that are displayed publicly are
    /// retrieved. For hidden or deleted episodes, use [`episode()`](Webtoon::episode()).
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813564).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// let episodes = webtoon.episodes().await?;
    /// println!("Total episodes: {}", episodes.count());
    ///
    /// for episode in episodes {
    ///     println!("Episode title: {}", episode.title().await?);
    ///
    ///     // Episodes behind cookies do not have a published date.
    ///     if let Some(published) = episode.published().await? {
    ///         println!("Published at: {}", published);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # All Episodes with [`episode()`](Webtoon::episode())
    ///
    /// If wanting to get all episodes, up to the latest released episode, including deleted or hidden episodes, you can
    /// should use `episode()` instead:
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838956).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// // Episode numbering starts at `1`.
    /// let mut number = 1;
    ///
    /// while let Some(episode) = webtoon.episode(number).await? {
    ///     println!("Episode title: {}", episode.title().await?);
    ///     number += 1;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn episodes(&self) -> Result<Episodes, EpisodeError> {
        let mut episodes = Vec::new();

        let response = self.client.get_episodes_json(self, 1, Sort::Asc).await?;

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
            let response = self.client.get_episodes_json(self, page, Sort::Asc).await?;

            let txt = response.text().await?;

            let json: Root = serde_json::from_str(&txt) //
                .map_err(|err| EpisodeError::Unexpected(err.into()))?;

            for article in json.article_list {
                episodes.push(Episode::from((self.clone(), article)));
            }
        }

        // 1, 2, 3, 4, ...
        episodes.sort_unstable();

        let episodes = Episodes {
            count: episodes.len() as u16,
            episodes,
        };

        Ok(episodes)
    }

    /// Constructs an [`Episode`], if it exists.
    ///
    /// # Caveats
    ///
    /// - **Episode Existence vs. Public Display**:
    ///   - This method can include episodes that are unpublished, behind ads or fast-pass, or even "deleted" (i.e., episodes that no longer appear on the main page but are still accessible through their episode number).
    ///   - It does not rely solely on public episodes, meaning it will retrieve episodes that a regular user would not normally see without having access to a creator's dashboard.
    ///   - The numbering of episodes retrieved by this method may differ from public episode lists due to the inclusion of hidden or removed episodes. You can see the matching episode with the `no=` query in the URL.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(730656).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(42).await? {
    ///     println!("Episode title: {}", episode.title().await?);
    /// } else {
    ///     println!("`{}` does not have an episode `42`", webtoon.title());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Behavior
    ///
    /// - This method sums the likes across all episodes, regardless of whether the episodes are:
    ///   - **Free**: It includes public episodes that any user can see.
    ///   - **Paid**: Episodes behind fast-pass or ad walls.
    ///   - **Deleted or Hidden**: Episodes that no longer appear on the public main page but still exist in the system.
    ///
    /// # Discrepancy
    ///
    /// This including those behind ads, fast-pass, or even deleted episodes. This can lead to a discrepancy between the publicly displayed episodes and the actual total likes, as it accounts for episodes that are normally hidden or restricted from public view.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838432).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// println!("`{}` has `{} total likes!`", webtoon.title(), webtoon.likes().await?);
    ///
    /// # Ok(())
    /// # }
    /// ```
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
    /// This can include posts from not yet free and hidden or deleted episodes.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(838432).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// for comment in webtoon.posts().await? {
    ///    println!("{}: \"{}\"", comment.poster().username(), comment.body());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
