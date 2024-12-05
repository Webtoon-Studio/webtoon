//! Represents an abstraction for the `https://www.webtoons.com/*/canvas/list?genreTab=ALL&sortOrder=` endpoint.

pub(super) mod episodes;
pub(super) mod info;
pub(super) mod likes;
pub(super) mod posts;
// pub mod search;

use super::{
    // canvas::{self, Sort},
    creator::Creator,
    errors::{
        CanvasError, ClientError, CreatorError, OriginalsError, PostError, SearchError,
        WebtoonError,
    },
    // originals::{self},
    webtoon::episode::{
        posts::{Post, Reaction},
        Episode,
    },
    Type,
    Webtoon,
};
use anyhow::{anyhow, Context};
use info::Info;
use reqwest::Response;
// use search::Item;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, env, ops::RangeBounds, str::FromStr, sync::Arc};
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
/// # use webtoon::platform::naver::Client;
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
    pub async fn creator(&self, id: u64) -> Result<Option<Creator>, CreatorError> {
        todo!()
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
    async fn search(&self, query: &str) -> Result<Vec<()>, SearchError> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        todo!()
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
    async fn originals(&self) -> Result<Vec<Webtoon>, OriginalsError> {
        // originals::scrape(self).await
        todo!()
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
    async fn canvas(
        &self,
        pages: impl RangeBounds<u16> + Send,
        // sort: Sort,
    ) -> Result<Vec<Webtoon>, CanvasError> {
        // canvas::scrape(self, pages, sort).await
        todo!()
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
    /// # use webtoon::platform::naver::{errors::Error, Type, Client};
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
    pub async fn webtoon(&self, id: u32) -> Result<Option<Webtoon>, WebtoonError> {
        if let Some(info) = self.webtoon_info(id).await? {
            return Ok(Some(Webtoon {
                client: self.clone(),
                id: info.id,
                r#type: info.r#type,
                creators: info
                    .creators
                    .into_iter()
                    .map(|creator| Creator::from((creator, self)))
                    .collect(),
                title: info.title,
                genres: info.data.genres,
                favorites: info.favorites,
                summary: info.summary,
                thumbnail: info.thumbnail,
                weekdays: info.weekdays,
                is_completed: info.is_completed,
            }));
        };

        Ok(None)
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
    pub async fn webtoon_from_url(&self, url: &str) -> Result<Webtoon, WebtoonError> {
        todo!()
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
    async fn has_valid_session(&self) -> Result<bool, ClientError> {
        todo!()
    }
}

// Internal only impls
impl Client {
    pub(super) async fn webtoon_info(&self, id: u32) -> Result<Option<Info>, ClientError> {
        let url = format!("https://comic.naver.com/api/article/list/info?titleId={id}");

        let response = self.http.get(url).send().await?;

        if response.status() != 200 {
            eprintln!("response was not 200: {response:#?}");
            return Ok(None);
        }

        let text = response.text().await?;

        let info: Info = serde_json::from_str(&text).context(text)?;

        Ok(Some(info))
    }

    pub(super) async fn favorite_webtoon(&self, webtoon: &Webtoon) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn unfavorite_webtoon(&self, webtoon: &Webtoon) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn rate_webtoon(
        &self,
        webtoon: &Webtoon,
        rating: u8,
    ) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn likes_for_webtoon(&self, episode: &Episode) -> Result<u32, ClientError> {
        let id = episode.webtoon.id;
        let url = format!(
            "https://comic.like.naver.com/v1/search/contents?pq=COMIC[p_{id}]&q=COMIC[{id}_1]"
        );

        let response = self
            .http
            .get(url)
            .send()
            .await?
            .json::<likes::Response>()
            .await?;

        let likes = response
            .parent_contents
            .first()
            .unwrap()
            .parent_reactions
            .first()
            .unwrap()
            .count;

        Ok(likes)
    }

    pub(super) async fn likes_for_episode(&self, episode: &Episode) -> Result<u32, ClientError> {
        let id = episode.webtoon.id;
        let number = episode.number();
        let url = format!("https://comic.like.naver.com/v1/search/contents?pq=COMIC[p_{id}]&q=COMIC[{id}_{number}]");

        let response = self
            .http
            .get(url)
            .send()
            .await?
            .json::<likes::Response>()
            .await?;

        let likes = response
            .contents
            .first()
            .unwrap()
            .reactions
            .first()
            // NOTE: If there are no likes for an episode then this will be `None`, even if episode exits.
            .map(|count| count.count)
            .unwrap_or_default();

        Ok(likes)
    }

    pub(super) async fn user_action(
        &self,
        episode: &Episode,
    ) -> Result<Option<episodes::rating::UserAction>, ClientError> {
        let id = episode.webtoon.id;
        let number = episode.number();
        let url = format!("https://comic.naver.com/api/userAction/info?titleId={id}&no={number}");

        let response = self.http.get(url).send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let text = response.text().await?;

        let user_action =
            serde_json::from_str(&text).map_err(|err| ClientError::Unexpected(err.into()))?;

        Ok(Some(user_action))
    }

    pub(super) async fn like_episode(&self, episode: &Episode) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn unlike_episode(&self, episode: &Episode) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn posts_for_episode_at_page(
        &self,
        episode: &Episode,
        page: u32,
        stride: u8,
    ) -> Result<posts::Response, ClientError> {
        let id = episode.webtoon.id;
        let number = episode.number();

        let url = format!("https://apis.naver.com/commentBox/cbox/web_naver_list_jsonp.json?ticket=comic&pool=cbox3&lang=ko&pageSize={stride}&sort=new&objectId={id}_{number}&page={page}");

        let response = self
            .http
            .get(url)
            .header("Referer", "https://comic.naver.com/")
            .send()
            .await?;

        let text = response.text().await?;

        let response =
            serde_json::from_str(text.trim_start_matches("_callback(").trim_end_matches(");"))
                .context(text)?;

        Ok(response)
    }

    pub(super) async fn webtoon_episodes_at_page(
        &self,
        webtoon: &Webtoon,
        page: u8,
    ) -> Result<episodes::Response, ClientError> {
        let id = webtoon.id();

        let url = format!("https://comic.naver.com/api/article/list?titleId={id}&page={page}");

        let response = self
            .http
            .get(url)
            .send()
            .await?
            .json::<episodes::Response>()
            .await?;

        Ok(response)
    }

    pub(super) async fn get_upvotes_and_downvotes_for_post(
        &self,
        post: &Post,
    ) -> Result<Response, ClientError> {
        todo!()
    }

    pub(super) async fn get_replies_for_post(
        &self,
        post: &Post,
        stride: u8,
    ) -> Result<Response, ClientError> {
        todo!()
    }

    pub(super) async fn post_reply(
        &self,
        post: &Post,
        body: &str,
        is_spoiler: bool,
    ) -> Result<(), ClientError> {
        todo!()
    }

    pub(super) async fn delete_post(&self, post: &Post) -> Result<(), PostError> {
        todo!()
    }

    pub(super) async fn put_react_to_post(&self, post: &Post) -> Result<(), PostError> {
        todo!()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
