//! Represents a client abstraction for `webtoons.com`.

mod api;

// TODO: Is this the best spot for this to be exported?
pub use api::user_info::UserInfo;

use crate::{
    platform::webtoons::{
        client::api::{
            api_token::ApiToken,
            creator_webtoons::CreatorWebtoons,
            dashboard::episodes::DashboardEpisode,
            likes::RawLikesResponse,
            posts::{Count, RawPostResponse},
            react_token::ReactToken,
            webtoon_user_info::WebtoonUserInfo,
        },
        error::{
            ApiTokenError, ClientBuilderError, CreatorWebtoonsError, DeletePostError, EpisodeError,
            InvalidWebtoonUrl, LikesError, PostsError, ReactTokenError, RequestError, SessionError,
            UserInfoError,
        },
        search::Item,
        webtoon::post::{PinRepresentation, id::Id},
    },
    stdx::{
        error::{Invariant, invariant},
        http::{DEFAULT_USER_AGENT, IRetry},
    },
};

use super::{
    Language, Type, Webtoon,
    canvas::{self, Sort},
    creator::{self, Creator},
    error::{
        CanvasError, ClientError, CreatorError, OriginalsError, PostError, SearchError,
        WebtoonError,
    },
    meta::Scope,
    originals::{self},
    webtoon::{
        episode::Episode,
        post::{Post, Reaction},
    },
};
use parking_lot::RwLock;
use scraper::Html;
use serde_json::json;
use std::{collections::HashMap, ops::RangeBounds, sync::Arc};

#[cfg(feature = "rss")]
use std::str::FromStr;

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
/// # Ok::<(), webtoon::platform::webtoons::error::ClientBuilderError>(())
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
    /// # use webtoon::platform::webtoons::{ClientBuilder, error::ClientBuilderError, Client};
    /// let client: Client = ClientBuilder::new().build()?;
    /// # Ok::<(), webtoon::platform::webtoons::error::ClientBuilderError>(())
    /// ```
    pub fn build(self) -> Result<Client, ClientBuilderError> {
        Ok(Client {
            http: self
                .builder
                .build()
                .map_err(|_err| ClientBuilderError::BuildFailed)?,
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
        #[expect(
            clippy::expect_used,
            reason = "it is documented that this can panic and that `ClientBuilder` should be used instead for a `Result`"
        )]
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
        #[expect(
            clippy::expect_used,
            reason = "it is documented that this can panic and that `ClientBuilder` should be used instead for a `Result`"
        )]
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
    /// # use webtoon::platform::webtoons::{Client, Language, error::{Error, CreatorError}};
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

        let Some(page) = creator::homepage(language, profile, self).await? else {
            return Ok(None);
        };

        Ok(Some(Creator {
            client: self.clone(),
            language,
            profile: Some(profile.into()),
            username: page.username.clone(),
            homepage: Arc::new(RwLock::new(Some(page))),
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
    /// # use webtoon::platform::webtoons::{Client, Language, error::Error};
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

        let mut webtoons = Vec::with_capacity(100);

        let language = match language {
            Language::En => "ENGLISH",
            Language::Zh => "TRADITIONAL_CHINESE",
            Language::Th => "THAI",
            Language::Id => "INDONESIAN",
            Language::Es => "SPANISH",
            Language::Fr => "FRENCH",
            Language::De => "GERMAN",
        };

        // Originals
        {
            // `nextSize` max is 50. Anything else is a BAD_REQUEST.
            // `contentSubType`:
            // - ALL
            // - CHALLENGE
            // - WEBTOON
            let url = format!(
                "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={language}&query={query}"
            );

            let response = self
                .http
                .get(&url)
                .retry()
                .send()
                .await
                .map_err(RequestError)?;

            let json = response.text().await.map_err(RequestError)?;

            let search = match serde_json::from_str::<api::search::RawSearch>(&json) {
                Ok(search) => search,
                Err(err) => invariant!(
                    "failed to deserialize `webtoon.com` originals search api response (structure change possible): {err}\n\n{json}"
                ),
            };

            let Some(originals) = search.result.webtoon_title_list else {
                invariant!(
                    "search result didnt have `webtoonTitleList`(originals) field in response result"
                );
            };

            // Initial response.
            for data in originals.data {
                let webtoon = Item {
                    client: self.clone(),
                    id: data.content_id,
                    r#type: Type::Original,
                    title: data.name,
                    thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                    creator: data.extra.writer.nickname,
                };

                webtoons.push(webtoon);
            }

            // Rest.
            let mut next = originals.pagination.next;
            while let Some(ref cursor) = next {
                let url = format!(
                    "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=WEBTOON&nextSize=50&language={language}&query={query}&cursor={cursor}"
                );

                let response = self
                    .http
                    .get(&url)
                    .retry()
                    .send()
                    .await
                    .map_err(RequestError)?;

                let json = response.text().await.map_err(RequestError)?;

                let search = match serde_json::from_str::<api::search::RawSearch>(&json) {
                    Ok(search) => search,
                    Err(err) => invariant!(
                        "failed to deserialize `webtoon.com` originals search api response (structure change possible): {err}\n\n{json}"
                    ),
                };

                let Some(originals) = search.result.webtoon_title_list else {
                    invariant!(
                        "search result didnt have `webtoonTitleList`(originals) field in response result"
                    );
                };

                for data in originals.data {
                    let webtoon = Item {
                        client: self.clone(),
                        id: data.content_id,
                        r#type: Type::Original,
                        title: data.name,
                        thumbnail: format!(
                            "https://swebtoon-phinf.pstatic.net{}",
                            data.thumbnail.path
                        ),
                        creator: data.extra.writer.nickname,
                    };

                    webtoons.push(webtoon);
                }
                next = originals.pagination.next;
            }
        }

        // Canvas
        {
            let url = format!(
                "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={language}&query={query}"
            );

            let response = self
                .http
                .get(&url)
                .retry()
                .send()
                .await
                .map_err(RequestError)?;

            let json = response.text().await.map_err(RequestError)?;

            let search = match serde_json::from_str::<api::search::RawSearch>(&json) {
                Ok(search) => search,
                Err(err) => invariant!(
                    "failed to deserialize `webtoon.com` canvas search api response (structure change possible): {err}\n\n{json}"
                ),
            };

            let Some(canvas) = search.result.challenge_title_list else {
                invariant!(
                    "search result didnt have `challengeTitleList`(canvas) field in response result"
                );
            };

            // Initial response.
            for data in canvas.data {
                let webtoon = Item {
                    client: self.clone(),
                    id: data.content_id,
                    r#type: Type::Canvas,
                    title: data.name,
                    thumbnail: format!("https://swebtoon-phinf.pstatic.net{}", data.thumbnail.path),
                    creator: data.extra.writer.nickname,
                };

                webtoons.push(webtoon);
            }

            // Rest.
            let mut next = canvas.pagination.next;
            while let Some(ref cursor) = next {
                let url = format!(
                    "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType=CHALLENGE&nextSize=50&language={language}&query={query}&cursor={cursor}"
                );

                let response = self
                    .http
                    .get(&url)
                    .retry()
                    .send()
                    .await
                    .map_err(RequestError)?;

                let json = response.text().await.map_err(RequestError)?;

                let search = match serde_json::from_str::<api::search::RawSearch>(&json) {
                    Ok(search) => search,
                    Err(err) => invariant!(
                        "failed to deserialize `webtoon.com` originals search api response (structure change possible): {err}\n\n{json}"
                    ),
                };

                let Some(canvas) = search.result.challenge_title_list else {
                    invariant!(
                        "search result didnt have `challengeTitleList`(canvas) field in response result"
                    );
                };

                for data in canvas.data {
                    let webtoon = Item {
                        client: self.clone(),
                        id: data.content_id,
                        r#type: Type::Canvas,
                        title: data.name,
                        thumbnail: format!(
                            "https://swebtoon-phinf.pstatic.net{}",
                            data.thumbnail.path
                        ),
                        creator: data.extra.writer.nickname,
                    };

                    webtoons.push(webtoon);
                }
                next = canvas.pagination.next;
            }
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
    /// # use webtoon::platform::webtoons::{ Client, Language, error::Error};
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
    /// # use webtoon::platform::webtoons::{ Client, Language, error::Error, canvas::Sort};
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
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
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
        Webtoon::new_with_client(id, r#type, self).await
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
    /// # use webtoon::platform::webtoons::{error::Error, Client};
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
    pub fn webtoon_from_url(&self, url: &str) -> Result<Webtoon, InvalidWebtoonUrl> {
        Webtoon::from_url_with_client(url, self)
    }

    /// Returns a [`UserInfo`] derived from a passed in session.
    ///
    /// This can be useful if you need to get the profile or username from the session alone.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client};
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
    pub async fn user_info_for_session(&self, session: &str) -> Result<UserInfo, SessionError> {
        let response = self
            .http
            .get("https://www.webtoons.com/en/member/userInfo")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<UserInfo>(&response) {
            Ok(user_info) => Ok(user_info),
            Err(err) => {
                invariant!(
                    "failed to deserialize `userInfo` from `webtoons.com` response: {err}\n\n{response}"
                )
            }
        }
    }

    /// Returns if the `Client` was provided a session.
    ///
    /// This does **NOT** mean session is valid.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
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
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    /// assert!(!client.has_valid_session().await?);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_valid_session(&self) -> Result<bool, SessionError> {
        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided);
        };

        let user_info = self.user_info_for_session(session).await?;

        Ok(user_info.is_logged_in())
    }
}

// Internal only impls
impl Client {
    pub(super) async fn get_originals_page(
        &self,
        language: Language,
        day: &str,
    ) -> Result<Html, ClientError> {
        let language = match language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };

        let url = format!("https://www.webtoons.com/{language}/originals/{day}");

        let document = self
            .http
            .get(&url)
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        let html = Html::parse_document(&document);

        Ok(html)
    }

    pub(super) async fn get_canvas_page(
        &self,
        language: Language,
        page: u16,
        sort: Sort,
    ) -> Result<Html, ClientError> {
        let language = match language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };

        let url = format!(
            "https://www.webtoons.com/{language}/canvas/list?genreTab=ALL&sortOrder={sort}&page={page}"
        );

        let document = self
            .http
            .get(&url)
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        let html = Html::parse_document(&document);

        Ok(html)
    }

    pub(super) async fn get_creator_page(
        &self,
        language: Language,
        profile: &str,
    ) -> Result<Option<Html>, CreatorError> {
        let language = match language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };

        let url = format!("https://www.webtoons.com/p/community/{language}/u/{profile}");

        let response = self
            .http
            .get(&url)
            .retry()
            .send()
            .await
            .map_err(RequestError)?;

        if response.status() == 404 {
            return Ok(None);
        }

        if response.status() == 400 {
            return Err(CreatorError::DisabledByCreator);
        }

        let document = response.text().await.map_err(RequestError)?;

        let html = Html::parse_document(&document);

        Ok(Some(html))
    }

    pub(super) async fn get_creator_webtoons(
        &self,
        profile: &str,
        language: Language,
    ) -> Result<CreatorWebtoons, CreatorWebtoonsError> {
        let language = match language {
            Language::En => "ENGLISH",
            Language::Zh => "TRADITIONAL_CHINESE",
            Language::Th => "THAI",
            Language::Id => "INDONESIAN",
            Language::Es => "SPANISH",
            Language::Fr => "FRENCH",
            Language::De => "GERMAN",
        };

        let url = format!(
            "https://www.webtoons.com/p/community/api/v1/creator/{profile}/titles?language={language}"
        );

        // TODO: return specific error if profile is not the correct profile to use.
        // This will be matched on by the caller.
        let response = self.http.get(url).send().await.map_err(RequestError)?;

        let json = response.text().await.map_err(RequestError)?;

        match serde_json::from_str::<CreatorWebtoons>(&json) {
            Ok(creator_webtoons) => Ok(creator_webtoons),
            Err(err) => {
                invariant!(
                    "failed to deserialize creator webtoons `webtoons.com` response: {err}\n\n{json}"
                )
            }
        }
    }

    pub(super) async fn get_webtoon_page(
        &self,
        webtoon: &Webtoon,
        page: Option<u16>,
    ) -> Result<Html, ClientError> {
        let id = webtoon.id;
        let language = match webtoon.language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };
        let scope = webtoon.scope.as_slug();
        let slug = &webtoon.slug;

        let url = if let Some(page) = page {
            format!(
                "https://www.webtoons.com/{language}/{scope}/{slug}/list?title_no={id}&page={page}"
            )
        } else {
            format!("https://www.webtoons.com/{language}/{scope}/{slug}/list?title_no={id}")
        };

        let response = self
            .http
            .get(&url)
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        let html = Html::parse_document(&response);

        Ok(html)
    }

    pub(super) async fn post_subscribe_to_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<(), SessionError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession);
        }

        let session = self.session.as_ref().invariant(
            "checked with `has_valid_session` that session should exist, so should never be `None`",
        )?;

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
            .await
            .map_err(RequestError)?;

        Ok(())
    }

    pub(super) async fn post_unsubscribe_to_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<(), SessionError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession);
        }

        let session = self.session.as_ref().invariant(
            "checked with `has_valid_session` that session should exist, so should never be `None`",
        )?;

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
            .await
            .map_err(RequestError)?;

        Ok(())
    }

    pub(super) async fn get_episodes_dashboard(
        &self,
        webtoon: &Webtoon,
        page: u16,
    ) -> Result<Vec<DashboardEpisode>, EpisodeError> {
        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided.into());
        };

        let url = format!(
            "https://www.webtoons.com/*/challenge/dashboardEpisode?titleNo={id}&page={page}",
            id = webtoon.id
        );

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        api::dashboard::episodes::parse(&response)
    }

    pub(super) async fn get_stats_dashboard(
        &self,
        webtoon: &Webtoon,
    ) -> Result<Html, SessionError> {
        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided);
        };

        let language = match webtoon.language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };
        let scope = match webtoon.scope {
            Scope::Canvas => "challenge",
            Scope::Original(_) => "*",
        };
        let id = webtoon.id;

        let url = format!(r"https://www.webtoons.com/{language}/{scope}/titleStat?titleNo={id}");

        let response = self
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        let html = Html::parse_document(&response);

        Ok(html)
    }

    #[cfg(feature = "rss")]
    pub(super) async fn get_rss_for_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<rss::Channel, WebtoonError> {
        let id = webtoon.id;
        let language = match webtoon.language {
            Language::En => "en",
            Language::Zh => "zh-hant",
            Language::Th => "th",
            Language::Id => "id",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
        };
        let scope = match webtoon.scope {
            Scope::Original(genre) => genre.as_slug(),
            Scope::Canvas => "challenge",
        };
        let slug = &webtoon.slug;

        let url = format!("https://www.webtoons.com/{language}/{scope}/{slug}/rss?title_no={id}");

        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match rss::Channel::from_str(&response) {
            Ok(channel) => Ok(channel),
            Err(err) => invariant!("rss feed returned from `webtoons.com` failed to parse: {err}"),
        }
    }

    pub(super) async fn get_episode(
        &self,
        webtoon: &Webtoon,
        episode: u16,
    ) -> Result<Html, EpisodeError> {
        let id = webtoon.id;
        let scope = webtoon.scope.as_slug();

        // Language isn't needed
        let url = format!(
            "https://www.webtoons.com/*/{scope}/*/*/viewer?title_no={id}&episode_no={episode}"
        );

        let response = self
            .http
            .get(&url)
            .retry()
            .send()
            .await
            .map_err(RequestError)?;

        if response.status() == 404 {
            return Err(EpisodeError::NotViewable);
        }

        let document = response.text().await.map_err(RequestError)?;

        let html = Html::parse_document(&document);

        Ok(html)
    }

    pub(super) async fn get_likes_for_episode(
        &self,
        episode: &Episode,
    ) -> Result<RawLikesResponse, LikesError> {
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
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<RawLikesResponse>(&response) {
            Ok(response) => Ok(response),
            Err(err) => invariant!(
                "failed to deserialize raw likes api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    pub(super) async fn like_episode(&self, episode: &Episode) -> Result<(), SessionError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession);
        }

        let session = self
            .session
            .as_ref()
            // TODO: This is already checked on `has_valid_session`.
            // Try to encapsulate returning a Option valid session ans return Invalid
            // on None
            .ok_or(SessionError::NoSessionProvided)?;

        let webtoon = episode.webtoon.id;
        let r#type = episode.webtoon.scope.as_single_letter();
        let number = episode.number;

        let response = self.get_react_token().await?;

        // TODO: there is some logic overlap with `unlike_episode` that could be consolidated in `get_react_token`
        if response.success {
            let token = response
                .result
                .guest_token
                .invariant("if `webtoons.com` react token api response is successful, the `guestToken` should be Some")?;

            let timestamp = response
                .result
                .timestamp
                .invariant("if `webtoons.com` react token api response is successful, the `timestamp` should be Some")?;

            let language = match episode.webtoon.language {
                Language::En => "en",
                Language::Zh => "zh-hant",
                Language::Th => "th",
                Language::Id => "id",
                Language::Es => "es",
                Language::Fr => "fr",
                Language::De => "de",
            };

            let url = format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .post(&url)
                .header("Cookie", format!("NEO_SES={session}"))
                .retry()
                .send()
                .await
                .map_err(RequestError)?;
        }

        Ok(())
    }

    pub(super) async fn unlike_episode(&self, episode: &Episode) -> Result<(), SessionError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession);
        }

        let session = self
            .session
            .as_ref()
            .ok_or(SessionError::NoSessionProvided)?;

        let webtoon = episode.webtoon.id;
        let r#type = episode.webtoon.scope.as_single_letter();
        let number = episode.number;

        let response = self.get_react_token().await?;

        if response.success {
            let token = response
                .result
                .guest_token
                .invariant("if `webtoons.com` react token api response is successful, the `guestToken` should be Some")?;

            let timestamp = response
                .result
                .timestamp
                .invariant("if `webtoons.com` react token api response is successful, the `timestamp` should be Some")?;

            let language = match episode.webtoon.language {
                Language::En => "en",
                Language::Zh => "zh-hant",
                Language::Th => "th",
                Language::Id => "id",
                Language::Es => "es",
                Language::Fr => "fr",
                Language::De => "de",
            };

            let url = format!(
                "https://www.webtoons.com/api/v1/like/services/LINEWEBTOON/contents/{type}_{webtoon}_{number}?menuLanguageCode={language}&timestamp={timestamp}&guestToken={token}"
            );

            self.http
                .delete(&url)
                .header("Cookie", format!("NEO_SES={session}"))
                .retry()
                .send()
                .await
                .map_err(RequestError)?;
        }

        Ok(())
    }

    pub(super) async fn get_posts_for_episode(
        &self,
        episode: &Episode,
        cursor: Option<Id>,
        stride: u8,
        pin_representation: PinRepresentation,
    ) -> Result<RawPostResponse, PostsError> {
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

        let url = match pin_representation {
            PinRepresentation::None => format!(
                "https://www.webtoons.com/p/api/community/v2/posts?pageId={scope}_{webtoon}_{episode}&pinRepresentation=none&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=true"
            ),
            // Adds `is_top/isPinned` info for posts.
            PinRepresentation::Distinct => format!(
                "https://www.webtoons.com/p/api/community/v1/page/{scope}_{webtoon}_{episode}/posts/search?pinRepresentation=distinct&prevSize=0&nextSize=1"
            ),
        };

        let response = self
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<RawPostResponse>(&response) {
            Ok(response) => Ok(response),
            Err(err) => invariant!(
                "failed to deserialize raw post api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    pub(super) async fn check_if_episode_exists(
        &self,
        episode: &Episode,
    ) -> Result<bool, ClientError> {
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
            "https://www.webtoons.com/p/api/community/v2/posts?pageId={scope}_{webtoon}_{episode}&pinRepresentation=none&prevSize=0&nextSize=1&cursor=&withCursor=true"
        );

        let response = self
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?;

        Ok(response.status() != 404)
    }

    pub(super) async fn get_upvotes_and_downvotes_for_post(
        &self,
        post: &Post,
    ) -> Result<Count, PostError> {
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
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<Count>(&response) {
            Ok(count) => Ok(count),
            Err(err) => invariant!(
                "failed to deserialize post upvote/downvote api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    pub(super) async fn get_replies_for_post(
        &self,
        post: &Post,
        cursor: Option<Id>,
        stride: u8,
    ) -> Result<RawPostResponse, PostsError> {
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
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<RawPostResponse>(&response) {
            Ok(response) => Ok(response),
            Err(err) => invariant!(
                "failed to deserialize raw post api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    pub(super) async fn post_reply(
        &self,
        post: &Post,
        body: &str,
        is_spoiler: bool,
    ) -> Result<(), SessionError> {
        // TODO: validate session
        let session = self
            .session
            .as_deref()
            .ok_or(SessionError::NoSessionProvided)?;

        let page_id = format!(
            "{}_{}_{}",
            match post.episode.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            post.episode.webtoon.id,
            post.episode.number
        );

        let spoiler_filter = if is_spoiler { "ON" } else { "OFF" };
        let body = json![
            {
                "pageId": page_id,
                "parentId": post.id.to_string(),
                "settings": { "reply": "OFF", "reaction": "ON", "spoilerFilter": spoiler_filter },
                "title":"",
                "body": body
            }
        ];

        let token = self.get_api_token().await?;

        self.http
            .post("https://www.webtoons.com/p/api/community/v2/post")
            .json(&body)
            .header("Api-Token", token.clone())
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await
            .map_err(RequestError)?;

        // TODO: validate response was successful

        Ok(())
    }

    pub(super) async fn delete_post(&self, post: &Post) -> Result<(), DeletePostError> {
        let token = self.get_api_token().await?;

        let session = self
            .session
            .as_ref()
            .map(|session| session.as_ref())
            .ok_or(SessionError::NoSessionProvided)?;

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
            .await
            .map_err(RequestError)?;

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
            .ok_or(SessionError::NoSessionProvided)?;

        self.http
            .put(&url)
            .header("Service-Ticket-Id", "epicom")
            .header("Referer", "https://www.webtoons.com/")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Api-Token", token.clone())
            .retry()
            .send()
            .await
            .map_err(RequestError)?;

        Ok(())
    }

    pub(super) async fn get_user_info_for_webtoon(
        &self,
        webtoon: &Webtoon,
    ) -> Result<WebtoonUserInfo, UserInfoError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession.into());
        }

        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided.into());
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
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<WebtoonUserInfo>(&response) {
            Ok(response) => Ok(response),
            Err(err) => invariant!(
                "failed to deserialize webtoon user info api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    async fn get_react_token(&self) -> Result<ReactToken, ReactTokenError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession.into());
        }

        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided.into());
        };

        let response = self
            .http
            .get("https://www.webtoons.com/api/v1/like/react-token")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Referer", "https://www.webtoons.com")
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<ReactToken>(&response) {
            Ok(token) => Ok(token),
            Err(err) => invariant!(
                "failed to deserialize react token api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }

    pub(super) async fn get_api_token(&self) -> Result<String, ApiTokenError> {
        if !self.has_valid_session().await? {
            return Err(SessionError::InvalidSession.into());
        }

        let Some(session) = &self.session else {
            return Err(SessionError::NoSessionProvided.into());
        };

        let response = self
            .http
            .get("https://www.webtoons.com/p/api/community/v1/api-token")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await
            .map_err(RequestError)?
            .text()
            .await
            .map_err(RequestError)?;

        match serde_json::from_str::<ApiToken>(&response) {
            Ok(response) => Ok(response.result.token),
            Err(err) => invariant!(
                "failed to deserialize api token api response from `webtoons.com` response: {err}\n\n{response}"
            ),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
