//! Represents an abstraction for the `https://www.webtoons.com/*/canvas/list?genreTab=ALL&sortOrder=` endpoint.

pub(super) mod likes;
pub(super) mod posts;
pub mod search;

use super::{
    canvas::{self, Sort},
    creator::{self, Creator},
    errors::{
        CanvasError, ClientError, CreatorError, OriginalsError, PostError, SearchError,
        WebtoonError,
    },
    meta::Scope,
    originals::{self},
    webtoon::episode::{
        posts::{Post, Reaction},
        Episode,
    },
    Language, Type, Webtoon,
};
use anyhow::{anyhow, Context};
use posts::id::Id;
use reqwest::Response;
use search::Item;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, env, ops::RangeBounds, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A builder for configuring and creating instances of [`Client`] with custom settings.
///
/// The `ClientBuilder` provides an API for fine-tuning various aspects of the `Client`
/// configuration, such as session tokens, and custom user agents. It enables a more controlled construction
/// of the `Client` when the default configuration isn't sufficient.
///
/// ### Usage
///
/// The builder allows for method chaining to incrementally configure the client, with the final
/// step being a call to `.build()`, which consumes the builder and returns a `Client`.
///
/// ### Example
///
/// ```rust
/// # use webtoon::platform::webtoons::ClientBuilder;
/// let client = ClientBuilder::new()
///     .with_session("session-token")
///     .user_agent("custom-agent/1.0")
///     .build()
///     .expect("Failed to create Client");
/// ```
///
/// ### Notes
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
    /// This includes a default user agent (`webtoon/VERSION`).
    /// This is the starting point for configuring a `Client`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let builder = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
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
    /// This method is useful when creating a `Client` that needs to make authenticated requests.
    /// The session token will be included in all subsequent requests made by the resulting `Client`.
    ///
    /// ### Parameters
    ///
    /// - `session`: A string reference representing the session token.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new().with_session("session-token");
    /// ```
    ///
    /// ### Returns
    ///
    /// Returns the modified `ClientBuilder` with the session token set.
    #[must_use]
    pub fn with_session(mut self, session: &str) -> Self {
        self.session = Some(Arc::from(session));
        self
    }

    /// Sets a custom `User-Agent` header for the `Client`.
    ///
    /// Use this method when you want to specify a different `User-Agent` string for your API requests.
    /// By default, the user agent is set to `webtoon/VERSION`, but this can be overridden for
    /// custom implementations.
    ///
    /// ### Parameters
    ///
    /// - `user_agent`: A string reference representing the custom user agent.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new().user_agent("custom-agent/1.0");
    /// ```
    ///
    /// ### Returns
    ///
    /// Returns the modified `ClientBuilder` with the custom `User-Agent` set.
    #[must_use]
    pub fn user_agent(self, user_agent: &str) -> Self {
        let builder = self.builder.user_agent(user_agent);

        Self { builder, ..self }
    }

    /// Consumes the `ClientBuilder` and returns a fully-configured `Client`.
    ///
    /// This method finalizes the configuration of the `ClientBuilder` and attempts to build
    /// a `Client` based on the current settings. If there are issues with the underlying
    /// configuration (e.g., TLS backend failure or resolver issues), an error is returned.
    ///
    /// ### Errors
    ///
    /// This method returns a [`ClientError`] if the underlying HTTP client could not be built,
    /// such as when TLS initialization fails or the DNS resolver cannot load the system configuration.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let client = ClientBuilder::new().build().expect("Failed to build Client");
    /// ```
    ///
    /// ### Returns
    ///
    /// A `Result` containing the configured `Client` on success, or a `ClientError` on failure.
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

/// A high-level asynchronous client to interact with the `webtoons.com` API.
///
/// The `Client` is designed for efficient, reusable HTTP interactions, and internally
/// manages connection pooling for optimal performance. This means that a single `Client`
/// instance can be reused across multiple API calls without additional setup.
///
/// ### Configuration
///
/// Default settings for the `Client` are tuned for general usage scenarios, but you can
/// customize the behavior by utilizing the `Client::builder()` method, which provides
/// advanced configuration options.
///
/// ### Example
///
/// ```rust
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
    /// Instantiates a new [`Client`] without an active session, using the default user agent `webtoon/VERSION`.
    ///
    /// This method configures a basic [`Client`] with standard settings. If no session is needed or if default
    /// configurations are sufficient, this is the simplest way to create a `Client`.
    ///
    /// ### Panics
    ///
    /// This function will panic if the TLS backend cannot be initialized or if the DNS resolver
    /// fails to load the system's configuration. For a safer alternative that returns a `Result`
    /// instead of panicking, consider using the [`ClientBuilder`] for more controlled error handling.
    ///
    /// ### Example
    ///
    /// ```rust
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
    /// ### Parameters
    ///
    /// - `session`: A reference to the session token string. This will be stored internally and
    ///   used for any subsequent API requests that need a session.
    ///
    /// ### Panics
    ///
    /// Similar to `Client::new`, this method will panic if the TLS backend or DNS resolver cannot be initialized.
    /// For a non-panicking version, use the [`ClientBuilder`] for custom error handling.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::Client;
    /// let client = Client::with_session("my-session-token");
    /// ```
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
    /// You can specify other options by chaining methods on the builder before finalizing it with `.build()`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::Client;
    /// let client = Client::builder()
    ///     .with_session("my-session-token")
    ///     .build()
    ///     .expect("Failed to build client");
    /// ```
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

// Public facing impls
impl Client {
    /// Fetches the creator profile page for a given user in the specified language, returning a [`Creator`].
    ///
    /// This method attempts to retrieve the creator's profile page based on the `profile` and
    /// `language` provided. However, not all languages have a creator page on Webtoons.com,
    /// and this method accounts for such cases.
    ///
    /// ### Supported & Unsupported Languages
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
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::{Error, CreatorError}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// match client.creator("_profile", Language::En).await {
    ///     Ok(Some(creator)) => println!("Creator found: {:?}", creator),
    ///     Ok(None) => println!("No creator found for this profile."),
    ///     Err(CreatorError::UnsupportedLanguage) => println!("This language does not support creator profiles."),
    ///     Err(err) => println!("An error occurred: {:?}", err),
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
            page: Arc::new(Mutex::new(Some(page))),
        }))
    }

    /// Searches for webtoons on Webtoons.com based on a query string and language.
    ///
    /// This method performs a search on the Webtoons platform using the provided query string and language.
    /// It returns a list of `Item` objects that match the search criteria.
    ///
    /// ### Parameters
    ///
    /// - `search`: A `&str` representing the search query (e.g., a partial or full title of a webtoon or creator).
    /// - `language`: A [`Language`] enum value that determines the language version of Webtoons to search on.
    ///   Only webtoons available in the specified language will be included in the results.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Vec<Item>, SearchError>` containing:
    ///
    /// - `Ok(Vec<Item>)`: A vector of `Item` objects that match the search query in the specified language.
    /// - `Err(SearchError)`: An error if the search request fails (e.g., due to network issues or a rate limit being exceeded).
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// let search = client.search("Monsters And", Language::En).await?;
    /// for item in search {
    ///     println!("Webtoon: {}", item.title());
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// ### Notes
    ///
    /// - The search query is case-insensitive and will return webtoons that partially or fully match the provided string.
    /// - The search is specific to the language provided in the `language` parameter. Only webtoons available in the chosen language will be returned.
    ///
    /// ### Language Support
    ///
    /// The `Language` enum includes several languages (e.g., En, Zh, Es, etc.). The corresponding language code is mapped to
    /// the Webtoons API’s expected format for search queries.
    ///
    /// ### Errors
    ///
    /// - `SearchError::ParseError`: An error encountered during the parsing of the search results (to be implemented).
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
        let url = format!("https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={lang}&query={query}");

        let response = self.http.get(url).send().await?;

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
            let url = format!("https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={lang}&query={query}&cursor={cursor}");
            let response = self.http.get(url).send().await?;

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

        let url = format!("https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={lang}&query={query}");

        let response = self.http.get(url).send().await?;

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
            let url = format!("https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={lang}&query={query}&cursor={cursor}");
            let response = self.http.get(url).send().await?;

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

    /// Retrieves a list of all "Original" webtoons for the specified language from Webtoons.com.
    ///
    /// ### Language Support
    ///
    /// The `originals` section of the Webtoons site is available in different languages, and
    /// the `language` parameter allows you to specify which language version of the site to
    /// scrape. This determines the set of webtoons returned in the list.
    ///
    /// ### Parameters
    ///
    /// - `language`: The language in which to scrape the list of original webtoons. This must be a valid
    ///   [`Language`] enum value supported by Webtoons (e.g., En, Es).
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Vec<Webtoon>, OriginalsError>` containing:
    ///
    /// - `Ok(Vec<Webtoon>)`: A vector of `Webtoon` objects representing the original webtoons
    ///   available in the given language.
    /// - `Err(OriginalsError)`: An error if the scraping process fails (e.g., due to a network issue
    ///   or unexpected issue when parsing the html to Webtoon handles).
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// let originals = client.originals(Language::En).await?;
    /// for webtoon in originals {
    ///     println!("Webtoon: {:?}", webtoon.title().await?);
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// ### Notes
    ///
    /// The list of original webtoons can vary between languages, as each language version of the site
    /// may have different exclusive series. Ensure that the `Language` value provided corresponds to
    /// a valid section of the Webtoons site.
    pub async fn originals(&self, language: Language) -> Result<Vec<Webtoon>, OriginalsError> {
        originals::scrape(self, language).await
    }

    /// Retrieves a list of "Canvas" webtoons for the specified language from Webtoons.com,
    /// with support for pagination and sorting options.
    ///
    /// ### Language Support
    ///
    /// The `canvas` section is available in multiple languages, and the `language` parameter
    /// determines which language version of the site is scraped.
    ///
    /// ### Pagination and Sorting
    ///
    /// You can specify which pages to scrape using the `pages` parameter, which accepts any
    /// valid range (e.g., `1..5` for pages 1 through 4). The `sort` parameter allows you to
    /// control how the results are ordered:
    ///
    /// - `Sort::Popularity`: Orders by read count.
    /// - `Sort::Likes`: Orders by the number of likes.
    /// - `Sort::Date`: Orders by the most recent updates.
    ///
    /// ### Parameters
    ///
    /// - `language`: The language in which to scrape the list of Canvas webtoons. Must be a valid
    ///   [`Language`] enum value supported by Webtoons (e.g., En, Es).
    /// - `pages`: A range specifying which pages to scrape. This can be a range like `1..=3`
    ///   (pages 1 through 3) or `..10` (up to page 9).
    /// - `sort`: Specifies the order in which the webtoons should be retrieved. Must be a valid
    ///   [`Sort`] enum value (e.g., `Sort::Popularity`, `Sort::Likes`, `Sort::Date`).
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Vec<Webtoon>, CanvasError>` containing:
    ///
    /// - `Ok(Vec<Webtoon>)`: A vector of `Webtoon` objects representing the Canvas webtoons
    ///   available in the given language, sorted and paginated based on the provided parameters.
    /// - `Err(CanvasError)`: An error if the scraping process fails (e.g., due to network issues
    ///   or unexpected problems when parsing the HTML to Webtoon handles).
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, errors::Error, canvas::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// let webtoons = client
    ///     .canvas(Language::En, 1..=3, Sort::Popularity)
    ///     .await?;
    ///
    /// for webtoon in webtoons {
    ///     println!("Webtoon: {}", webtoon.title().await?);
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// ### Notes
    ///
    /// The list of Canvas webtoons can vary between languages, and the sorting order may impact
    /// the results significantly. Ensure that the `Language` and `Sort` values match the intended
    /// criteria for your search.
    ///
    /// Due to limitations of how Webtoons responds to the request, there is no way to know if the page requested exists(No more pages).
    /// In the interest of sane defaults, an unbounded end is equal to `..100`. If not, this function would never return.
    pub async fn canvas(
        &self,
        language: Language,
        pages: impl RangeBounds<u16> + Send,
        sort: Sort,
    ) -> Result<Vec<Webtoon>, CanvasError> {
        canvas::scrape(self, language, pages, sort).await
    }

    /// Constructs a `Webtoon` from the given `id` and `type`.
    ///
    /// ### Parameters
    ///
    /// - `id`: The unique ID of the webtoon to retrieve.
    /// - `type`: Specifies the type of the webtoon—either `Original` or `Canvas`.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Option<Webtoon>, WebtoonError>` containing:
    ///
    /// - `Ok(Some(Webtoon))`: A `Webtoon` object representing the webtoon.
    /// - `Ok(None)`: If the webtoon does not exist (HTTP 404).
    /// - `Err(WebtoonError)`: An error if something goes wrong during the request or URL parsing process.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// if let Some(webtoon) = client.webtoon(123456, Type::Original).await? {
    ///     println!("Webtoon ID: {}, Language: {:?}", webtoon.id(), webtoon.language());
    /// } else {
    ///     println!("Webtoon does not exist.");
    /// }
    /// # Ok(())}
    /// ```
    pub async fn webtoon(&self, id: u32, r#type: Type) -> Result<Option<Webtoon>, WebtoonError> {
        let url = format!(
            "https://www.webtoons.com/*/{}/*/list?title_no={id}",
            match r#type {
                Type::Original => "*",
                Type::Canvas => "canvas",
            }
        );

        let response = self.http.get(&url).send().await?;

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
            page: Arc::new(Mutex::new(None)),
        };

        Ok(Some(webtoon))
    }

    /// Constructs a `Webtoon` from a given URL.
    ///
    /// ### URL Structure
    ///
    /// The provided URL must follow the typical structure used by Webtoons.com webtoons:
    ///
    /// - `https://www.webtoons.com/{language}/{scope}/{slug}/list?title_no={id}`
    ///
    /// It is assumed that the webtoon will always exist given the URL. This simplfies usage and cleans up boilerplate.
    ///  Errors during the request or parsing process are returned as `WebtoonError`.
    ///
    /// ### Parameters
    ///
    /// - `url`: A reference to the webtoon's URL (`&str`) from which the webtoon information will be parsed.
    ///
    /// ### Returns
    ///
    /// Returns a `Result<Webtoon, WebtoonError>` containing:
    ///
    /// - `Ok(Webtoon)`: A `Webtoon` object.
    /// - `Err(WebtoonError)`: An error if something goes wrong during the request or URL parsing process.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// let webtoon = client
    ///     .webtoon_from_url("https://www.webtoons.com/en/action/omniscient-reader/list?title_no=2154")?;
    ///
    /// println!("Webtoon ID: {}",  webtoon.title().await?);
    /// # Ok(())}
    /// ```
    ///
    /// ### Notes
    ///
    /// - The URL must be a valid Webtoons.com URL, otherwise the function will return a `WebtoonError`.
    /// - The method expects the `title_no` query parameter to be present in the URL, as this is how the webtoon ID is identified.
    ///
    /// ### Errors
    ///
    /// - If the URL cannot be parsed, the method will return an appropriate `WebtoonError` describing the failure.
    /// - The method also handles other potential parsing issues, such as missing path segments or invalid query parameters.
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
            page: Arc::new(Mutex::new(None)),
        };

        Ok(webtoon)
    }

    /// Returns user info derived from the passed in session.
    ///
    /// This can be useful if you need to get the profile or username from the session alone.
    ///
    /// # Errors
    ///
    /// Will return an error if there was an issue with the network request or deserilization.
    pub async fn user_info_for_session(&self, session: &str) -> Result<UserInfo, ClientError> {
        let mut count = 5;

        let text = loop {
            eprintln!("count");
            if count == 0 {
                return Err(ClientError::RateLimitExceeded);
            }

            let response = self
                .http
                .get("https://www.webtoons.com/en/member/userInfo")
                .header("Cookie", format!("NEO_SES={session}"))
                .send()
                .await?
                .text()
                .await?;

            count -= 1;

            if !response.contains("429 Too Many Requests") {
                break response;
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        };

        let user_info: UserInfo = serde_json::from_str(&text).map_err(|err| {
            ClientError::Unexpected(anyhow!("failed to deserialize `userInfo` endpoint: {err}"))
        })?;

        Ok(user_info)
    }

    /// Returns if the client was provided a session.
    ///
    /// This does **NOT** mean session is valid.
    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    /// Tries to validate the current session.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the session is valid.
    /// - `Ok(false)` if the session is not valid.
    ///
    /// # Note
    ///
    /// This is mainly provided for a quick early return for when the circumstances allow, and any methods that use a
    /// session should not rely on the session always being valid after this check; The session could be invalidated
    /// after the check completes.
    ///
    /// # Errors
    ///
    /// [`ClientError::NoSessionProvided`] if session was never proivided.
    /// [`ClientError::Unexpected`] if there was an error in request or deserialization.
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
    pub(super) async fn get_originals_page(&self, lang: Language) -> Result<Response, ClientError> {
        let url = format!("https://www.webtoons.com/{lang}/originals");
        let response = self.http.get(url).send().await?;
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

        let response = self.http.get(url).send().await?;

        Ok(response)
    }

    pub(super) async fn get_creator_page(
        &self,
        lang: Language,
        profile: &str,
    ) -> Result<Response, ClientError> {
        let url = format!("https://www.webtoons.com/p/community/{lang}/u/{profile}");
        let response = self.http.get(url).send().await?;
        Ok(response)
    }

    pub(super) async fn get_webtoon_page(
        &self,
        webtoon: &Webtoon,
        page: Option<u8>,
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

        let response = self.http.get(url).send().await?;

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
            .send()
            .await?;

        Ok(())
    }

    pub(super) async fn post_rate_webtoon(
        &self,
        webtoon: &Webtoon,
        rating: u8,
    ) -> Result<(), ClientError> {
        if !self.has_valid_session().await? {
            return Err(ClientError::InvalidSession);
        };

        let url = match webtoon.scope {
            Scope::Original(_) => "https://www.webtoons.com/setStarScore",
            Scope::Canvas => "https://www.webtoons.com/canvas/setStarScore",
        };

        let mut form = HashMap::new();
        form.insert("titleNo", webtoon.id.to_string());
        form.insert("score", rating.to_string());

        let session = self.session.as_ref().unwrap();

        self.http
            .post(url)
            .header("Referer", "https://www.webtoons.com/")
            // NOTE: `wtu` just has to have something as a value and it works
            .header("Cookie", format!("NEO_SES={session}; wtu=WTU"))
            .form(&form)
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
            .get(url)
            .header("Cookie", format!("NEO_SES={session}"))
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
            .get(url)
            .header("Cookie", format!("NEO_SES={session}"))
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

        let response = self.http.get(url).send().await?;

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

        self.http
            .get(url)
            .header("Cookie", format!("NEO_SES={session}"))
            .send()
            .await
            .map_err(|err| ClientError::Unexpected(err.into()))
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

            let url =  format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .post(url)
                .header("Cookie", format!("NEO_SES={session}"))
                .send()
                .await
                .map_err(|err| ClientError::Unexpected(err.into()))?;
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

            let url =  format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .delete(url)
                .header("Cookie", format!("NEO_SES={session}"))
                .send()
                .await
                .map_err(|err| ClientError::Unexpected(err.into()))?;
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

        let url = format!("https://www.webtoons.com/p/api/community/v2/posts?pageId={scope}_{webtoon}_{episode}&pinRepresentation=none&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=true");

        self.http
            .get(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .send()
            .await
            .map_err(|err| ClientError::Unexpected(err.into()))
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

        let url =format!("https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/count", post.id) ;

        let response = self
            .http
            .get(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
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

        let url = format!("https://www.webtoons.com/p/api/community/v2/post/{post_id}/child-posts?sort=oldest&displayBlindCommentAsService=false&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=false");

        let response = self
            .http
            .get(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
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
            .header("Api-Token", token)
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Service-Ticket-Id", "epicom")
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
            .header("Api-Token", token)
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Service-Ticket-Id", "epicom")
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
            Reaction::Upvote => format!("https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/like", post.id),
            Reaction::Downvote => format!("https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/dislike", post.id),
            Reaction::None => unreachable!("Should never be used with `Reaction::None`"),
        };

        let token = self.get_api_token().await?;

        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .ok_or(ClientError::NoSessionProvided)?;

        self.http
            .put(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Referer", "https://www.webtoons.com/")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Api-Token", token)
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
            .send()
            .await?
            .text()
            .await?;

        let title_user_info = serde_json::from_str(&response).context(response)?;

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
            .send()
            .await?
            .text()
            .await?;

        let api_token = serde_json::from_str::<ReactToken>(&response).context(response)?;

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
            .send()
            .await?
            .text()
            .await?;

        let api_token = serde_json::from_str::<ApiToken>(&response).context(response)?;

        Ok(api_token.result.token)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns data from the `webtoons.com/en/member/userInfo` URL.
///
/// This can be used to get the username and profile, as well as check if user is logged in.
#[derive(Deserialize, Debug)]
pub struct UserInfo {
    #[serde(rename = "loginUser")]
    is_logged_in: bool,

    #[serde(rename = "nickname")]
    username: String,

    #[serde(rename = "profileUrl")]
    profile: String,
}

impl UserInfo {
    /// Returns if current user session is logged in.
    ///
    /// Functionally this tells whether a session is valid or not.
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// Returns webtoons username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the profile segment for webtoons.com/*/creator/{profile}
    pub fn profile(&self) -> &str {
        &self.profile
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebtoonUserInfo {
    author: bool,
    pub(super) favorite: bool,
    pub(super) star_score: Option<u8>,
}

impl WebtoonUserInfo {
    pub fn is_webtoon_creator(&self) -> bool {
        self.author
    }

    #[allow(unused)]
    pub fn did_rate(&self) -> bool {
        self.favorite
    }

    /// If no rating was given, this will return `None`.
    #[allow(unused)]
    pub fn rating_given(&self) -> Option<u8> {
        self.star_score
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
