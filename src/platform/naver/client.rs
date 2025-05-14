//! Represents a client abstraction for `comic.naver.com`, both public and private methods.

pub(super) mod episodes;
mod info;
pub(super) mod likes;
pub(super) mod posts;
pub(super) mod rating;

use crate::stdx::{
    http::{DEFAULT_USER_AGENT, IRetry},
    math::MathExt,
};

use super::{
    Type, Webtoon,
    creator::{self, Creator},
    errors::{ClientError, CreatorError, WebtoonError},
    meta::Genre,
    webtoon::{
        WebtoonInner,
        episode::{Episode, posts::Post},
    },
};
use anyhow::Context;
use episodes::Sort;
use info::Info;
use parking_lot::RwLock;
use reqwest::{Response, redirect::Policy};
use std::{str::FromStr, sync::Arc};
use url::Url;

const EPISODES_PER_PAGE: u16 = 20;

/// A builder for configuring and creating instances of [`Client`] with custom settings.
///
/// The `ClientBuilder` provides an API for fine-tuning various aspects of the `Client`
/// configuration and custom user agents. It enables a more controlled construction
/// of the `Client` when the default configuration isn't sufficient.
///
/// # Usage
///
/// The builder allows for method chaining to incrementally configure the client, with the final
/// step being a call to [`build()`](ClientBuilder::build()), which consumes the builder and returns a [`Client`].
///
/// # Example
///
/// ```
/// # use webtoon::platform::naver::ClientBuilder;
/// let client = ClientBuilder::new()
///     .user_agent("custom-agent/1.0")
///     .build()?;
/// # Ok::<(), webtoon::platform::naver::errors::ClientError>(())
/// ```
///
/// # Notes
///
/// This builder is the preferred way to create clients when needing custom configurations, and
/// should be used instead of `Client::new()` for more advanced setups.
#[derive(Debug)]
pub struct ClientBuilder {
    builder: reqwest::ClientBuilder,
    // TODO: This is only needed because `reqwest::ClientBuilder` isn't `Clone`
    // and thus we cannot just clone when needed to build when used and change the user agent when needed.
    //
    // The user agent is only changed for `get_episode_page_html`.
    user_agent: Option<Arc<str>>,
}

impl Default for ClientBuilder {
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Creates a new `ClientBuilder` with default settings.
    ///
    /// This includes a default user agent (`webtoon/VERSION`), and is the starting point for configuring a `Client`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::ClientBuilder;
    /// let builder = ClientBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let builder = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .use_rustls_tls()
            .https_only(true)
            .brotli(true);

        Self {
            builder,
            user_agent: None,
        }
    }

    /// Sets a custom `User-Agent` header for the [`Client`].
    ///
    /// By default, the user agent is set to `webtoon/VERSION`, but this can be overridden using this method.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::ClientBuilder;
    /// let builder = ClientBuilder::new().user_agent("custom-agent/1.0");
    /// ```
    #[must_use]
    pub fn user_agent(self, user_agent: &str) -> Self {
        Self {
            user_agent: Some(user_agent.into()),
            builder: self.builder.user_agent(user_agent),
        }
    }

    /// Consumes the `ClientBuilder` and returns a fully-configured [`Client`].
    ///
    /// This method finalizes the configuration of the `ClientBuilder` and attempts to build
    /// a `Client` based on the current settings. If there are issues with the underlying
    /// configuration (e.g., TLS backend failure or resolver issues), an error is returned.
    ///
    /// # Errors
    ///
    /// This method returns a [`ClientError`] if the underlying HTTP client could not be built,
    /// such as when TLS initialization fails or the DNS resolver cannot load the system configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{ClientBuilder, Client};
    /// let client: Client = ClientBuilder::new().build()?;
    /// # Ok::<(), webtoon::platform::naver::errors::ClientError>(())
    /// ```
    pub fn build(self) -> Result<Client, ClientError> {
        Ok(Client {
            user_agent: self.user_agent.clone(),
            http: self
                .builder
                .build()
                .map_err(|err| ClientError::Unexpected(err.into()))?,
        })
    }
}

/// A high-level, asynchronous client to interact with `comic.naver.com`.
///
/// The `Client` is designed for efficient, reusable interactions, and internally
/// manages connection pooling for optimal performance.
///
/// # Configuration
///
/// Default settings for the `Client` are tuned for general usage scenarios, but you can
/// customize the behavior by utilizing the `Client::builder()` method, which provides
/// advanced configuration options.
///
/// # Example
///
/// ```
/// # use webtoon::platform::naver::Client;
/// let client = Client::new();
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(super) http: reqwest::Client,
    user_agent: Option<Arc<str>>,
}

// Creation impls
impl Client {
    /// Instantiates a new [`Client`] with the default user agent: `webtoon/VERSION`.
    ///
    /// This method configures a basic `Client` with standard settings. If default
    /// configurations are sufficient, this is the simplest way to create a `Client`.
    ///
    /// # Panics
    ///
    /// This function will panic if the TLS backend cannot be initialized or if the DNS resolver
    /// fails to load the system's configuration. For a safer alternative that returns a `Result`
    /// instead of panicking, consider using the [`ClientBuilder`] for more controlled error handling.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::Client;
    /// let client = Client::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        ClientBuilder::new().build().expect("Client::new()")
    }

    /// Returns a [`ClientBuilder`] for creating a custom-configured `Client`.
    ///
    /// The builder pattern allows for greater flexibility in configuring a `Client`.
    /// You can specify other options by chaining methods on the builder before finalizing it with [`ClientBuilder::build()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{Client, ClientBuilder};
    /// let builder: ClientBuilder = Client::builder();
    /// ```
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

// Public facing impls
impl Client {
    /// Fetches info for the Creator of a given `profile`.
    ///
    /// The `profile` can be found from the community page URL: [`https://comic.naver.com/community/u/_21cqqm`]
    ///
    /// **NOTE**: Not all Webtoon creators have a community page. This is usually denoted by green check mark next to
    /// their name on the Webtoon's page.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// match client.creator("_21cqqm").await {
    ///     Ok(Some(creator)) => println!("Creator found: {creator:?}"),
    ///     Ok(None) => unreachable!("profile is known to exist"),
    ///     Err(err) => panic!("An error occurred: {err:?}"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`https://comic.naver.com/community/u/_21cqqm`]: https://comic.naver.com/community/u/_21cqqm
    pub async fn creator(&self, profile: &str) -> Result<Option<Creator>, CreatorError> {
        let Some(page) = creator::page(profile, self).await? else {
            return Ok(None);
        };

        Ok(Some(Creator {
            client: self.clone(),
            profile: Some(profile.into()),
            username: page.username.clone(),
            page: Arc::new(RwLock::new(Some(page))),
        }))
    }

    /// Constructs a [`Webtoon`] from the given `id`.
    ///
    /// If no Webtoon is found for the given `id`, then `None` is returned.
    ///
    /// The id can be found from the URL query `titleId`: [`https://comic.naver.com/webtoon/list?titleId=832703`]
    ///
    /// # Platform Notes
    ///
    /// `comic.naver.com` id's are unique across the entire platform.
    ///
    /// # Panics
    ///
    /// There is an innate assumption that `comics.naver.com` only ever has valid URLs on its website. If this is broken,
    /// then this function can panic upon URL parsing.
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
    /// assert_eq!("시한부 천재가 살아남는 법", webtoon.title());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`https://comic.naver.com/webtoon/list?titleId=832703`]: https://comic.naver.com/webtoon/list?titleId=832703
    pub async fn webtoon(&self, id: u32) -> Result<Option<Webtoon>, WebtoonError> {
        let response = self.get_webtoon_json(id).await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let info: Info = serde_json::from_str(&response.text().await?) //
            .map_err(|err| WebtoonError::Unexpected(err.into()))?;

        let mut genres = Vec::new();

        for genre in info.gfp_ad_custom_param.genre_types {
            let genre = Genre::from_str(&genre) //
                .map_err(|err| WebtoonError::Unexpected(err.into()))?;

            genres.push(genre);
        }

        if genres.is_empty() {
            return Err(WebtoonError::NoGenre);
        }

        let mut creators = Vec::new();

        for creator in info.community_artists {
            let profile = match creator.profile_page_url {
                Some(url) => Url::parse(&url)
                    .expect("naver api should only have valid urls")
                    .path_segments()
                    .expect("url should have segments for the profile page")
                    .nth(2)
                    .map(|profile| profile.to_string()),
                None => None,
            };

            creators.push(Creator {
                client: self.clone(),
                username: creator.name,
                profile,
                page: Arc::new(RwLock::new(None)),
            });
        }

        let webtoon = Webtoon {
            inner: Arc::new(WebtoonInner {
                id,
                r#type: Type::from_str(&info.webtoon_level_code)?,
                title: info.title_name,
                summary: info.synopsis,
                thumbnail: info.shared_thumbnail_url,
                is_new: info.new,
                on_hiatus: info.rest,
                is_completed: info.finished,
                favorites: info.favorite_count,
                schedule: info.publish_day_of_week_list,
                genres,
                creators,
            }),

            client: self.clone(),
        };

        Ok(Some(webtoon))
    }

    /// Constructs a `Webtoon` from a given `url`.
    ///
    /// If no Webtoon is found for the given `url`, then `None` is returned.
    ///
    /// # URL Structure
    ///
    /// The provided URL must follow the typical structure used by `comic.naver.com` Webtoon's:
    ///
    /// - `https://comic.naver.com/webtoon/list?titleId={ID}`
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client
    ///     .webtoon_from_url("https://comic.naver.com/webtoon/list?titleId=838432").await? else {
    ///         unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!("우렉 마지노", webtoon.title());
    ///
    /// # Ok(())}
    /// ```
    pub async fn webtoon_from_url(&self, url: &str) -> Result<Option<Webtoon>, WebtoonError> {
        let url = url::Url::parse(url)?;

        let id = url
            .query_pairs()
            .find(|query| query.0 == "titleId")
            .ok_or(WebtoonError::InvalidUrl(
                "Naver URL should have a `titleId` query: failed to find one in provided URL.",
            ))?
            .1
            .parse::<u32>()
            .context("`titleId` query parameter wasn't able to parse into a u32")?;

        self.webtoon(id).await
    }
}

// Internal only impls
impl Client {
    pub(super) async fn get_creator_page(&self, profile: &str) -> Result<Response, ClientError> {
        let url = format!("https://comic.naver.com/community/u/{profile}");
        let response = self.http.get(&url).retry().send().await?;
        Ok(response)
    }

    pub(super) async fn get_webtoon_json(&self, id: u32) -> Result<Response, ClientError> {
        let url = format!("https://comic.naver.com/api/article/list/info?titleId={id}");
        let response = self.http.get(&url).retry().send().await?;
        Ok(response)
    }

    pub(super) async fn get_episode_page_html(
        &self,
        webtoon: &Webtoon,
        episode: u16,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id();
        let url = format!("https://comic.naver.com/webtoon/detail?titleId={id}&no={episode}");
        // NOTE: Cannot follow redirects as it will always return `200 OK`.
        // Need to see what the status is for the first hit.
        let client = reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .https_only(true)
            .brotli(true)
            .user_agent(
                self.user_agent
                    .as_ref()
                    .map_or(DEFAULT_USER_AGENT, |user_agent| &**user_agent),
            )
            .redirect(Policy::none())
            .build()
            .unwrap();

        let response = client.get(&url).retry().send().await?;
        Ok(response)
    }

    pub(super) async fn get_episodes_json(
        &self,
        webtoon: &Webtoon,
        page: u16,
        sort: Sort,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id();
        let url = format!(
            "https://comic.naver.com/api/article/list?titleId={id}&page={page}&sort={sort}"
        );
        let response = self.http.get(&url).retry().send().await?;
        Ok(response)
    }

    pub(super) async fn get_episode_info_from_json(
        &self,
        webtoon: &Webtoon,
        episode: u16,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id();
        let page = episode.in_bucket_of(EPISODES_PER_PAGE);

        let url =
            format!("https://comic.naver.com/api/article/list?titleId={id}&page={page}&sort=ASC");
        let response = self.http.get(&url).retry().send().await?;
        Ok(response)
    }

    pub(super) async fn get_likes_for_episode(
        &self,
        episode: &Episode,
    ) -> Result<Response, ClientError> {
        let id = episode.webtoon.id();
        let episode = episode.number;

        let url =
            format!("https://route-like.naver.com/v1/search/contents?q=COMIC[{id}_{episode}]");

        Ok(self.http.get(&url).retry().send().await?)
    }

    pub(super) async fn get_rating_for_episode(
        &self,
        episode: &Episode,
    ) -> Result<Response, ClientError> {
        let id = episode.webtoon.id();
        let episode = episode.number;
        let url = format!("https://comic.naver.com/api/userAction/info?titleId={id}&no={episode}");
        Ok(self.http.get(&url).retry().send().await?)
    }

    pub(super) async fn get_posts_for_episode(
        &self,
        episode: &Episode,
        page: u32,
        size: u32,
        sort: posts::Sort,
    ) -> Result<Response, ClientError> {
        let id = episode.webtoon.id();
        let episode = episode.number;

        let url = format!(
            "https://apis.naver.com/commentBox/cbox/web_naver_list_jsonp.json?ticket=comic&pool=cbox3&lang=ko&country=KR&objectId={id}_{episode}&pageSize={size}&indexSize=10&page={page}&sort={sort}"
        );

        let response = self
            .http
            .get(&url)
            .header("Referer", "https://comic.naver.com/")
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_replies_for_post(
        &self,
        post: &Post,
        page: u32,
    ) -> Result<Response, ClientError> {
        let parent_comment_number = &post.id;
        let id = post.episode.webtoon.id();
        let episode = post.episode.number();

        let url = format!(
            "https://apis.naver.com/commentBox/cbox/web_naver_list_jsonp.json?ticket=comic&pool=cbox3&lang=ko&country=KR&objectId={id}_{episode}&pageSize=100&indexSize=10&parentCommentNo={parent_comment_number}&page={page}&sort=NEW"
        );

        let response = self
            .http
            .get(&url)
            .header("Referer", "https://comic.naver.com/")
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_webtoons_from_creator_page(
        &self,
        profile: &str,
    ) -> Result<Response, ClientError> {
        let url = format!("https://comic.naver.com/community/api/v1/creator/{profile}/series");

        let response = self
            .http
            .get(&url)
            .header("Referer", "https://comic.naver.com/")
            .retry()
            .send()
            .await?;

        Ok(response)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
