//! Represents a client abstraction for `webtoons.com`.

pub(super) mod likes;
pub(super) mod posts;
pub mod search;

use crate::stdx::http::{DEFAULT_USER_AGENT, IRetry};

use super::{
    Language, Type, Webtoon,
    canvas::{self, Sort},
    creator::{self, Creator},
    errors::{
        CanvasError, ClientError, CreatorError, OriginalsError, PostError, SearchError,
        WebtoonError,
    },
    meta::Scope,
    originals::{self},
    webtoon::episode::{
        Episode,
        posts::{Post, Reaction},
    },
};
use anyhow::{Context, anyhow};
use parking_lot::RwLock;
use posts::id::Id;
use reqwest::Response;
use search::Item;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, ops::RangeBounds, str::FromStr, sync::Arc};

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
/// # use webtoon::platform::webtoons::ClientBuilder;
/// let client = ClientBuilder::new()
///     .user_agent("custom-agent/1.0")
///     .build()?;
/// # Ok::<(), webtoon::platform::webtoons::errors::ClientError>(())
/// ```
///
/// # Notes
///
/// This builder is the preferred way to create clients when needing custom configurations, and
/// should be used instead of `Client::new()` for more advanced setups.
#[derive(Debug)]
pub struct ClientBuilder {
    builder: reqwest::ClientBuilder,
    session: Option<Arc<str>>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Creates a new `ClientBuilder` with default settings.
    ///
    /// This includes a default user agent (`$CARGO_PKG_NAME/$CARGO_PKG_VERSION`), and is the starting point for configuring a `Client`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let builder = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .use_rustls_tls()
            .https_only(true)
            .brotli(true);

        Self {
            builder,
            session: None,
        }
    }

    /// Configures the `ClientBuilder` to use the specified session token for authentication.
    ///
    /// This method is useful when creating a `Client` that needs to make authenticated requests. The session token will
    /// be included in all subsequent requests made by the resulting `Client`, where needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new().with_session("session-token");
    /// ```
    #[inline]
    #[must_use]
    pub fn with_session(mut self, session: &str) -> Self {
        self.session = Some(Arc::from(session));
        self
    }

    /// Sets a custom `User-Agent` header for the [`Client`].
    ///
    /// By default, the user agent is set to (`$CARGO_PKG_NAME/$CARGO_PKG_VERSION`), but this can be overridden using this method.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new().user_agent("custom-agent/1.0");
    /// ```
    #[inline]
    #[must_use]
    pub fn user_agent(self, user_agent: &str) -> Self {
        let builder = self.builder.user_agent(user_agent);
        Self { builder, ..self }
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
    /// # use webtoon::platform::webtoons::{ClientBuilder, Client};
    /// let client: Client = ClientBuilder::new().build()?;
    /// # Ok::<(), webtoon::platform::webtoons::errors::ClientError>(())
    /// ```
    pub fn build(self) -> Result<Client, ClientError> {
        Ok(Client {
            http: self
                .builder
                .build()
                .map_err(|err| ClientError::Unexpected(err.into()))?,
            session: self.session,
        })
    }
}

/// A high-level, asynchronous client to interact with `webtoons.com`.
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
/// # use webtoon::platform::webtoons::Client;
/// let client = Client::new();
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(super) http: reqwest::Client,
    pub(super) session: Option<Arc<str>>,
}

// Creation impls
impl Client {
    /// Instantiates a new [`Client`] with the default user agent: (`$CARGO_PKG_NAME/$CARGO_PKG_VERSION`).
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
    /// # use webtoon::platform::webtoons::Client;
    /// let client = Client::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        ClientBuilder::new().build().expect("Client::new()")
    }

    /// Instantiates a new [`Client`] with a provided session token, allowing authenticated requests.
    ///
    /// Use this method when you have an active session that you wish to reuse for API calls requiring
    /// authentication. This allows the client to automatically include the session in requests.
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
    /// # use webtoon::platform::webtoons::Client;
    /// let client = Client::with_session("my-session-token");
    /// ```
    #[inline]
    #[must_use]
    pub fn with_session(session: &str) -> Self {
        ClientBuilder::new()
            .with_session(session)
            .build()
            .expect("Client::with_session()")
    }

    /// Returns a [`ClientBuilder`] for creating a custom-configured `Client`.
    ///
    /// The builder pattern allows for greater flexibility in configuring a `Client`.
    /// You can specify other options by chaining methods on the builder before finalizing it with [`ClientBuilder::build()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, ClientBuilder};
    /// let builder: ClientBuilder = Client::builder();
    /// ```
    #[inline]
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

// Public facing impls
impl Client {
    /// Fetches info for the [`Creator`] of a given `profile`.
    ///
    /// The `profile` can be found from the community page URL: [`https://www.webtoons.com/p/community/en/u/w7m5o`]
    ///
    /// **NOTE**: Not all Webtoon creators have a community page. This is usually denoted by green check mark next to
    /// their name on the Webtoon's page.
    ///
    /// # Supported & Unsupported Languages
    ///
    /// Some languages, such as French (`fr`), German (`de`), and Chinese (`zh-hant`), do not currently
    /// support creator pages. As a result, this method will return an error, specifically
    /// [`CreatorError::UnsupportedLanguage`], when a creator page is requested in these languages.
    ///
    /// For languages where a creator page is supported, the function returns an `Option<Creator>`:
    ///
    /// - `Ok(Some(creator))`: A valid creator profile page was found, and the returned `Creator`
    ///   can be used for further interactions.
    /// - `Ok(None)`: No creator profile page exists for the given `profile` in the selected
    ///   supported language. In this case, even though the language is supported, the creator
    ///   does not have a profile page.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Language, errors::{Error, CreatorError}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// match client.creator("w7m5o", Language::En).await {
    ///     Ok(Some(creator)) => println!("Creator found: {creator:?}"),
    ///     Ok(None) => unreachable!("profile is known to exist"),
    ///     Err(CreatorError::UnsupportedLanguage) => println!("This language does not support creator profiles."),
    ///     Err(err) => panic!("An error occurred: {err:?}"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`https://www.webtoons.com/p/community/en/u/w7m5o`]: https://www.webtoons.com/p/community/en/u/w7m5o
    pub async fn creator(
        &self,
        profile: &str,
        language: Language,
    ) -> Result<Option<Creator>, CreatorError> {
        if matches!(language, Language::Zh | Language::De | Language::Fr) {
            return Err(CreatorError::UnsupportedLanguage);
        }

        let Some(page) = creator::page(language, profile, self).await? else {
            return Ok(None);
        };

        Ok(Some(Creator {
            client: self.clone(),
            language,
            profile: Some(profile.into()),
            username: page.username.clone(),
            page: Arc::new(RwLock::new(Some(page))),
        }))
    }

    /// Searches for Webtoons on `webtoons.com`.
    ///
    /// This method performs a search on the Webtoons platform using the provided query string and [`Language`].
    /// It returns a list of [`Item`] that match the search criteria.
    ///
    /// # Notes
    ///
    /// - The search query is case-insensitive and will return webtoons that partially or fully match the provided string.
    /// - The search is specific to the language provided in the `language` parameter. Only webtoons available in the chosen language will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Language, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let search = client.search("Monsters And", Language::En).await?;
    ///
    /// for webtoon in search {
    ///     println!("Webtoon: {}", webtoon.title());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::too_many_lines)]
    pub async fn search(&self, query: &str, language: Language) -> Result<Vec<Item>, SearchError> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let mut webtoons = Vec::new();

        let lang = match language {
            Language::En => "ENGLISH",
            Language::Zh => "TRADITIONAL_CHINESE",
            Language::Th => "THAI",
            Language::Id => "INDONESIAN",
            Language::Es => "SPANISH",
            Language::Fr => "FRENCH",
            Language::De => "GERMAN",
        };

        // nextSize max is 50. Anything else is a BAD_REQUEST.
        // contentSubType:
        // - ALL
        // - CHALLENGE
        // - WEBTOON
        let url = format!(
            "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={lang}&query={query}"
        );

        let response = self.http.get(&url).retry().send().await?;

        let api = serde_json::from_str::<search::Api>(&response.text().await?)
            .context("Failed to deserialize search api response")?;

        let Some(originals) = api.result.webtoon_title_list else {
            return Err(SearchError::Unexpected(anyhow!(
                "Original search result didnt have `webtoonTitleList` field in result"
            )));
        };

        for data in originals.data {
            let id: u32 = data
                .content_id
                .parse()
                .context("Failed to parse webtoon id to u32")?;

            let webtoon = Item {
                client: self.clone(),
                id,
                r#type: Type::Original,
                title: data.name,
                thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                creator: data.extra.writer.nickname,
            };

            webtoons.push(webtoon);
        }

        let mut next = originals.pagination.next;
        while let Some(ref cursor) = next {
            let url = format!(
                "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={lang}&query={query}&cursor={cursor}"
            );

            let response = self.http.get(&url).retry().send().await?;

            let api = serde_json::from_str::<search::Api>(&response.text().await?)
                .context("Failed to deserialize search api response")?;

            let Some(originals) = api.result.webtoon_title_list else {
                return Err(SearchError::Unexpected(anyhow!(
                    "Original search result didnt have `webtoonTitleList` field in result"
                )));
            };

            for data in originals.data {
                let id: u32 = data
                    .content_id
                    .parse()
                    .context("Failed to parse webtoon id to u32")?;

                let webtoon = Item {
                    client: self.clone(),
                    id,
                    r#type: Type::Original,
                    title: data.name,
                    thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                    creator: data.extra.writer.nickname,
                };

                webtoons.push(webtoon);
            }
            next = originals.pagination.next;
        }

        let url = format!(
            "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={lang}&query={query}"
        );

        let response = self.http.get(&url).retry().send().await?;

        let api = serde_json::from_str::<search::Api>(&response.text().await?)
            .context("Failed to deserialize search api response")?;

        let Some(canvas) = api.result.challenge_title_list else {
            return Err(SearchError::Unexpected(anyhow!(
                "Canvas search result didnt have `challengeTitleList` field in result"
            )));
        };

        for data in canvas.data {
            let id: u32 = data
                .content_id
                .parse()
                .context("Failed to parse webtoon id to u32")?;

            let webtoon = Item {
                client: self.clone(),
                id,
                r#type: Type::Canvas,
                title: data.name,
                thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                creator: data.extra.writer.nickname,
            };

            webtoons.push(webtoon);
        }

        let mut next = canvas.pagination.next;
        while let Some(ref cursor) = next {
            let url = format!(
                "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={lang}&query={query}&cursor={cursor}"
            );

            let response = self.http.get(&url).retry().send().await?;

            let api = serde_json::from_str::<search::Api>(&response.text().await?)
                .context("Failed to deserialize search api response")?;

            let Some(canvas) = api.result.challenge_title_list else {
                return Err(SearchError::Unexpected(anyhow!(
                    "Canvas search result didnt have `challengeTitleList` field in result"
                )));
            };

            for data in canvas.data {
                let id: u32 = data
                    .content_id
                    .parse()
                    .context("Failed to parse webtoon id to u32")?;

                let webtoon = Item {
                    client: self.clone(),
                    id,
                    r#type: Type::Canvas,
                    title: data.name,
                    thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                    creator: data.extra.writer.nickname,
                };

                webtoons.push(webtoon);
            }
            next = canvas.pagination.next;
        }

        Ok(webtoons)
    }

    /// Retrieves a list of all `original` webtoons for the specified [`Language`] from `webtoons.com`.
    ///
    /// This corresponds to all webtoons found at `https://www.webtoons.com/*/originals`.
    ///
    /// # Language Support
    ///
    /// The `originals` section of the Webtoons site is available in different languages, and
    /// the `language` parameter allows you to specify which language version of the site to
    /// scrape. This determines the set of webtoons returned in the list.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let originals = client.originals(Language::En).await?;
    ///
    /// println!("Found {} originals", originals.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn originals(&self, language: Language) -> Result<Vec<Webtoon>, OriginalsError> {
        originals::scrape(self, language).await
    }

    /// Retrieves a list of `canvas` webtoons for the specified [`Language`] from `webtoons.com`.
    ///
    /// This corresponds to all webtoons found at `https://www.webtoons.com/*/canvas`.
    ///
    /// # Language Support
    ///
    /// The `canvas` section is available in multiple languages, and the `language` parameter
    /// determines which language version of the site is scraped.
    ///
    /// # Pagination and Sorting
    ///
    /// You can specify which pages to scrape using the `pages` parameter, which accepts any
    /// valid range (e.g., `1..5` for pages 1 through 4). The `sort` parameter allows you to
    /// control how the results are ordered:
    ///
    /// - `Sort::Popularity`: Orders by read count.
    /// - `Sort::Likes`: Orders by the number of likes.
    /// - `Sort::Date`: Orders by the most recent updates.
    ///
    /// # Notes
    ///
    /// The list of Canvas webtoons can vary between languages, and the sorting order may impact
    /// the results significantly.
    ///
    /// Due to limitations of how `webtoons.com` responds to the request, there is no way to know if the page requested
    /// exists(No more pages). In the interest of sane defaults, an unbounded end is equal to `..100`. If not, this function would never return.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::Error, canvas::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoons = client
    ///     .canvas(Language::En, 1..=3, Sort::Popularity)
    ///     .await?;
    ///
    /// for webtoon in webtoons {
    ///     println!("Webtoon: {}", webtoon.title().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn canvas(
        &self,
        language: Language,
        pages: impl RangeBounds<u16> + Send,
        sort: Sort,
    ) -> Result<Vec<Webtoon>, CanvasError> {
        canvas::scrape(self, language, pages, sort).await
    }

    /// Constructs a [`Webtoon`] from the given `id` and [`Type`].
    ///
    /// Both sides, `canvas` and `original`, have separate sets of `id`'s. This means that an original and a canvas story
    /// could have the same numerical id. The `id`'s are unique across languages though, so this method supports any language.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(95, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// assert_eq!("Tower of God", webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn webtoon(&self, id: u32, r#type: Type) -> Result<Option<Webtoon>, WebtoonError> {
        let url = format!(
            "https://www.webtoons.com/*/{}/*/list?title_no={id}",
            match r#type {
                Type::Original => "*",
                Type::Canvas => "canvas",
            }
        );

        let response = self.http.get(&url).retry().send().await?;

        // Webtoon doesn't exist
        if response.status() == 404 {
            return Ok(None);
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
            client: self.clone(),
            id,
            language,
            scope,
            slug: Arc::from(slug),
            page: Arc::new(RwLock::new(None)),
        };

        Ok(Some(webtoon))
    }

    /// Constructs a [`Webtoon`] from a given `url`.
    ///
    /// # URL Structure
    ///
    /// The provided `url` must follow the typical structure used by `webtoons.com`:
    ///
    /// - `https://www.webtoons.com/{language}/{scope}/{slug}/list?title_no={id}`
    ///
    /// It is assumed that the webtoon will always exist given the URL. This simplifies usage and cleans up boilerplate.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client
    ///     .webtoon_from_url("https://www.webtoons.com/en/action/omniscient-reader/list?title_no=2154")?;
    ///
    /// assert_eq!("Omniscient Reader",  webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn webtoon_from_url(&self, url: &str) -> Result<Webtoon, WebtoonError> {
        let url = url::Url::parse(url)?;

        let mut segments = url.path_segments().ok_or(WebtoonError::InvalidUrl(
            "Webtoon url should have segments separated by `/`; this url did not.",
        ))?;

        let segment = segments
            .next()
            .ok_or(WebtoonError::InvalidUrl(
                "Webtoon URL was found to have segments, but for some reason failed to extract that first segment, which should be a language code: e.g `en`",
            ))?;

        let language = Language::from_str(segment)
            .context("Failed to parse URL language code into `Language` enum")?;

        let segment = segments.next().ok_or(
                WebtoonError::InvalidUrl("Url was found to have segments, but didn't have a second segment, representing the scope of the webtoon.")
            )?;

        let scope = Scope::from_str(segment) //
            .context("Failed to parse URL scope path to a `Scope`")?;

        let slug = segments
            .next()
            .ok_or( WebtoonError::InvalidUrl( "Url was found to have segments, but didn't have a third segment, representing the slug name of the Webtoon."))?
            .to_string();

        let id = url
            .query()
            .ok_or(WebtoonError::InvalidUrl(
                "Webtoon URL should have a `title_no` query: failed to find one in provided URL.",
            ))?
            .split('=')
            .nth(1)
            .context("`title_no` should always have a `=` separator")?
            .parse::<u32>()
            .context("`title_no` query parameter wasn't able to parse into a u32")?;

        let webtoon = Webtoon {
            client: self.clone(),
            language,
            scope,
            slug: Arc::from(slug),
            id,
            page: Arc::new(RwLock::new(None)),
        };

        Ok(webtoon)
    }

    /// Returns a [`UserInfo`] derived from a passed in session.
    ///
    /// This can be useful if you need to get the profile or username from the session alone.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// // When no session, or an invalid session, is passed in, `is_logged_in()` will be false.
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert!(!user_info.is_logged_in());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn user_info_for_session(&self, session: &str) -> Result<UserInfo, ClientError> {
        let response = self
            .http
            .get("https://www.webtoons.com/en/member/userInfo")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        let user_info: UserInfo = serde_json::from_str(&response.text().await?).map_err(|err| {
            ClientError::Unexpected(anyhow!("failed to deserialize `userInfo` endpoint: {err}"))
        })?;

        Ok(user_info)
    }

    /// Returns if the `Client` was provided a session.
    ///
    /// This does **NOT** mean session is valid.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    /// assert!(!client.has_session());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    /// Tries to validate the current session.
    ///
    /// - `true` if the session is proven valid.
    /// - `false` if the session is proven invalid.
    ///
    /// <div class="warning">
    ///
    /// **This is mainly provided for a quick early return for when the circumstances allow. Any methods that use a
    /// session should not rely on the session always being valid after this check; The session could be invalidated
    /// after the check completes!**
    ///
    /// </div>
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    /// assert!(!client.has_valid_session().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_valid_session(&self) -> Result<bool, ClientError> {
        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let user_info = self.user_info_for_session(session).await?;

        Ok(user_info.is_logged_in)
    }
}

// Internal only impls
impl Client {
    pub(super) async fn get_originals_page(
        &self,
        lang: Language,
        day: &str,
    ) -> Result<Response, ClientError> {
        let url = format!("https://www.webtoons.com/{lang}/originals/{day}");
        let response = self.http.get(&url).retry().send().await?;

        Ok(response)
    }

    pub(super) async fn get_canvas_page(
        &self,
        lang: Language,
        page: u16,
        sort: Sort,
    ) -> Result<Response, ClientError> {
        let url = format!(
            "https://www.webtoons.com/{lang}/canvas/list?genreTab=ALL&sortOrder={sort}&page={page}"
        );

        let response = self.http.get(&url).retry().send().await?;

        Ok(response)
    }

    pub(super) async fn get_creator_page(
        &self,
        lang: Language,
        profile: &str,
    ) -> Result<Response, ClientError> {
        let url = format!("https://www.webtoons.com/p/community/{lang}/u/{profile}");

        let response = self.http.get(&url).retry().send().await?;

        Ok(response)
    }

    pub(super) async fn get_webtoon_page(
        &self,
        webtoon: &Webtoon,
        page: Option<u16>,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id;
        let lang = webtoon.language;
        let scope = webtoon.scope.as_slug();
        let slug = &webtoon.slug;

        let url = if let Some(page) = page {
            format!("https://www.webtoons.com/{lang}/{scope}/{slug}/list?title_no={id}&page={page}")
        } else {
            format!("https://www.webtoons.com/{lang}/{scope}/{slug}/list?title_no={id}")
        };

        let response = self.http.get(&url).retry().send().await?;

        Ok(response)
    }

    pub(super) async fn post_subscribe_to_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<(), ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let session = self.session.as_ref().unwrap();

        let mut form = HashMap::new();
        form.insert("titleNo", webtoon.id.to_string());
        form.insert("currentStatus", false.to_string());

        let url = match webtoon.scope {
            Scope::Original(_) => "https://www.webtoons.com/setFavorite",
            Scope::Canvas => "https://www.webtoons.com/challenge/setFavorite",
        };

        self.http
            .post(url)
            .header("Referer", "https://www.webtoons.com/")
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .form(&form)
            .retry()
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn post_unsubscribe_to_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<(), ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let session = self.session.as_ref().unwrap();

        let mut form = HashMap::new();
        form.insert("titleNo", webtoon.id.to_string());
        form.insert("currentStatus", true.to_string());

        let url = match webtoon.scope {
            Scope::Original(_) => "https://www.webtoons.com/setFavorite",
            Scope::Canvas => "https://www.webtoons.com/challenge/setFavorite",
        };

        self.http
            .post(url)
            .header("Referer", "https://www.webtoons.com/")
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .form(&form)
            .retry()
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn get_episodes_dashboard(
        &self,
        webtoon: &Webtoon,
        page: u16,
    ) -> Result<Response, ClientError> {
        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let id = webtoon.id;

        let url = format!(
            "https://www.webtoons.com/*/challenge/dashboardEpisode?titleNo={id}&page={page}"
        );

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_stats_dashboard(
        &self,
        webtoon: &Webtoon,
    ) -> Result<Response, ClientError> {
        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let lang = webtoon.language;
        let scope = match webtoon.scope {
            Scope::Canvas => "challenge",
            Scope::Original(_) => "*",
        };
        let id = webtoon.id;

        let url = format!(r"https://www.webtoons.com/{lang}/{scope}/titleStat?titleNo={id}");

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    #[cfg(feature = "rss")]
    pub(super) async fn get_rss_for_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id;
        let language = webtoon.language;
        let slug = &webtoon.slug;

        let scope = match webtoon.scope {
            Scope::Original(genre) => genre.as_slug(),
            Scope::Canvas => "challenge",
        };

        let url = format!("https://www.webtoons.com/{language}/{scope}/{slug}/rss?title_no={id}");

        let response = self.http.get(url).send().await?;

        Ok(response)
    }

    pub(super) async fn get_episode(
        &self,
        webtoon: &Webtoon,
        episode: u16,
    ) -> Result<Response, ClientError> {
        let id = webtoon.id;
        let scope = webtoon.scope.as_slug();

        // Language isn't needed
        let url = format!(
            "https://www.webtoons.com/*/{scope}/*/*/viewer?title_no={id}&episode_no={episode}"
        );

        let response = self.http.get(&url).retry().send().await?;

        Ok(response)
    }

    pub(super) async fn get_likes_for_episode(
        &self,
        episode: &Episode,
    ) -> Result<Response, ClientError> {
        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .unwrap_or_default();

        let scope = match episode.webtoon.scope {
            Scope::Original(_) => "w",
            Scope::Canvas => "c",
        };
        let webtoon = episode.webtoon.id;
        let episode = episode.number;

        let url = format!(
            "https://www.webtoons.com/api/v1/like/search/counts?serviceId=LINEWEBTOON&contentIds={scope}_{webtoon}_{episode}"
        );

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn like_episode(&self, episode: &Episode) -> Result<(), ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let session = self
            .session
            .as_ref()
            .ok_or(ClientError::NoSessionProvided)?;

        let webtoon = episode.webtoon.id;
        let r#type = episode.webtoon.scope.as_single_letter();
        let number = episode.number;

        let response = self.get_react_token().await?;

        if response.success {
            let token = response
                .result
                .guest_token
                .context("`guestToken` should be some if `success` is true")?;
            let timestamp = response
                .result
                .timestamp
                .context("`timestamp` should be some if `success` is true")?;

            let language = episode.webtoon.language;

            let url = format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .post(&url)
                .header("Cookie", format!("NEO_SES={session}"))
                .retry()
                .send()
                .await?;
        }

        Ok(())
    }

    pub(super) async fn unlike_episode(&self, episode: &Episode) -> Result<(), ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let session = self
            .session
            .as_ref()
            .ok_or(ClientError::NoSessionProvided)?;

        let webtoon = episode.webtoon.id;
        let r#type = episode.webtoon.scope.as_single_letter();
        let number = episode.number;

        let response = self.get_react_token().await?;

        if response.success {
            let token = response
                .result
                .guest_token
                .context("`guestToken` should be some if `success` is true")?;

            let timestamp = response
                .result
                .timestamp
                .context("`timestamp` should be some if `success` is true")?;

            let language = episode.webtoon.language;

            let url = format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .delete(&url)
                .header("Cookie", format!("NEO_SES={session}"))
                .retry()
                .send()
                .await?;
        }

        Ok(())
    }

    pub(super) async fn get_posts_for_episode(
        &self,
        episode: &Episode,
        cursor: Option<Id>,
        stride: u8,
    ) -> Result<Response, ClientError> {
        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .unwrap_or_default();

        let scope = match episode.webtoon.scope {
            Scope::Original(_) => "w",
            Scope::Canvas => "c",
        };

        let webtoon = episode.webtoon.id;

        let episode = episode.number;

        let cursor = cursor.map_or_else(String::new, |id| id.to_string());

        let url = format!(
            "https://www.webtoons.com/p/api/community/v2/posts?pageId={scope}_{webtoon}_{episode}&pinRepresentation=none&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=true"
        );

        let response = self
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_upvotes_and_downvotes_for_post(
        &self,
        post: &Post,
    ) -> Result<Response, ClientError> {
        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .unwrap_or_default();

        let page_id = format!(
            "{}_{}_{}",
            match post.episode.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            post.episode.webtoon.id,
            post.episode.number
        );

        let url = format!(
            "https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/count",
            post.id
        );

        let response = self
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_replies_for_post(
        &self,
        post: &Post,
        cursor: Option<Id>,
        stride: u8,
    ) -> Result<Response, ClientError> {
        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .unwrap_or_default();

        let post_id = post.id;

        let cursor = cursor.map_or_else(String::new, |id| id.to_string());

        let url = format!(
            "https://www.webtoons.com/p/api/community/v2/post/{post_id}/child-posts?sort=oldest&displayBlindCommentAsService=false&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=false"
        );

        let response = self
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        Ok(response)
    }

    pub(super) async fn post_reply(
        &self,
        post: &Post,
        body: &str,
        is_spoiler: bool,
    ) -> Result<(), ClientError> {
        let page_id = format!(
            "{}_{}_{}",
            match post.episode.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            post.episode.webtoon.id,
            post.episode.number
        );

        let parent_id = post.id.to_string();

        let spoiler_filter = if is_spoiler { "ON" } else { "OFF" };
        let body = json![
            {
                "pageId": page_id,
                "parentId": parent_id,
                "settings": { "reply": "OFF", "reaction": "ON", "spoilerFilter": spoiler_filter },
                "title":"",
                "body": body
            }
        ];

        let token = self.get_api_token().await?;

        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .ok_or(ClientError::NoSessionProvided)?;

        self.http
            .post("https://www.webtoons.com/p/api/community/v2/post")
            .json(&body)
            .header("Api-Token", token.clone())
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn delete_post(&self, post: &Post) -> Result<(), PostError> {
        let token = self.get_api_token().await?;

        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .ok_or(ClientError::NoSessionProvided)?;

        self.http
            .delete(format!(
                "https://www.webtoons.com/p/api/community/v2/post/{}",
                post.id
            ))
            .header("Api-Token", token.clone())
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn put_react_to_post(
        &self,
        post: &Post,
        reaction: Reaction,
    ) -> Result<(), PostError> {
        let page_id = format!(
            "{}_{}_{}",
            match post.episode.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            post.episode.webtoon.id,
            post.episode.number
        );

        let url = match reaction {
            Reaction::Upvote => format!(
                "https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/like",
                post.id
            ),
            Reaction::Downvote => format!(
                "https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/dislike",
                post.id
            ),
            Reaction::None => unreachable!("Should never be used with `Reaction::None`"),
        };

        let token = self.get_api_token().await?;

        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .ok_or(ClientError::NoSessionProvided)?;

        self.http
            .put(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Referer", "https://www.webtoons.com/")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Api-Token", token.clone())
            .retry()
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn get_user_info_for_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<WebtoonUserInfo, ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let url = match webtoon.scope {
            Scope::Original(_) => format!(
                "https://www.webtoons.com/getTitleUserInfo?titleNo={}",
                webtoon.id
            ),
            Scope::Canvas => {
                format!(
                    "https://www.webtoons.com/canvas/getTitleUserInfo?titleNo={}",
                    webtoon.id
                )
            }
        };

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        let text = response.text().await?;

        let title_user_info = serde_json::from_str(&text).context(text)?;

        Ok(title_user_info)
    }

    async fn get_react_token(&self) -> Result<ReactToken, ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let response = self
            .http
            .get("https://www.webtoons.com/api/v1/like/react-token")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Referer", "https://www.webtoons.com")
            .retry()
            .send()
            .await?;

        let text = response.text().await?;

        let api_token = serde_json::from_str::<ReactToken>(&text).context(text)?;

        Ok(api_token)
    }

    pub(super) async fn get_api_token(&self) -> Result<String, ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let Some(session) = &self.session else {
            return Err(ClientError::NoSessionProvided);
        };

        let response = self
            .http
            .get("https://www.webtoons.com/p/api/community/v1/api-token")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?;

        let text = response.text().await?;

        let api_token = serde_json::from_str::<ApiToken>(&text).context(text)?;

        Ok(api_token.result.token)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents data from the `webtoons.com/*/member/userInfo` endpoint.
///
/// This can be used to get the username and profile, as well as check if user is logged in. This type is not constructed
/// directly, but gotten through [`Client::user_info_for_session()`].
///
/// # Example
///
/// ```no_run
/// # use webtoon::platform::webtoons::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let user_info = client.user_info_for_session("session").await?;
///
/// assert!(!user_info.is_logged_in());
/// assert_eq!(Some("username"), user_info.username());
/// assert_eq!(Some("profile"), user_info.profile());
/// # Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug)]
pub struct UserInfo {
    #[serde(rename = "loginUser")]
    is_logged_in: bool,

    #[serde(rename = "nickname")]
    username: Option<String>,

    #[serde(rename = "profileUrl")]
    profile: Option<String>,
}

impl UserInfo {
    /// Returns if current user session is logged in.
    ///
    /// Functionally, this tells whether a session is valid or not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert!(!user_info.is_logged_in());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// Returns the users' username.
    ///
    /// If the session provided is invalid, then `username` will be `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert_eq!(Some("username"), user_info.username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns the profile segment for `webtoons.com/*/creator/{profile}`.
    ///
    /// If the session provided is invalid, then `profile` will be `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert_eq!(Some("profile"), user_info.profile());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebtoonUserInfo {
    author: bool,
    pub(super) favorite: bool,
}

impl WebtoonUserInfo {
    pub fn is_webtoon_creator(&self) -> bool {
        self.author
    }

    #[allow(unused)]
    pub fn did_rate(&self) -> bool {
        self.favorite
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub(super) struct ApiToken {
    status: String,
    result: Token,
}

#[derive(Deserialize, Debug)]
pub(super) struct Token {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReactToken {
    result: ReactResult,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactResult {
    guest_token: Option<String>,
    timestamp: Option<i64>,
    status_code: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewLikesResponse {
    result: NewLikesResult,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewLikesResult {
    count: u32,
}
