//! Represents a client abstraction for `webtoons.com`.

mod api;

// ASK: Is this the best spot for this to be exported?
pub use api::user_info::UserInfo;
use assumptions::{Assume, assume, assumption};

use super::{
    Type, Webtoon,
    canvas::{self, Sort},
    creator::{self, Creator},
    error::{CanvasError, CreatorError, OriginalsError, SearchError},
    originals::{self},
    webtoon::{episode::Episode, post::Post},
};
#[cfg(feature = "rss")]
use crate::platform::webtoons::error::WebtoonError;
use crate::{
    platform::webtoons::{
        client::api::{
            creator_webtoons::CreatorWebtoons,
            dashboard::{analytics::SeriesAnalytics, episodes::DashboardEpisode},
            likes::RawLikesResponse,
            posts::RawPostResponse,
            user_info::UserInfoRaw,
            webtoon_user_info::WebtoonUserInfo,
        },
        error::{
            ClientBuilderError, ClientError, CreatorWebtoonsError, EpisodeError, InvalidWebtoonUrl,
            LikesError, PostsError, SessionError, UserInfoError,
        },
        search::Item,
        webtoon::{
            Scope,
            post::{PinRepresentation, id::Id},
        },
    },
    stdx::{
        cache::Cache,
        http::{DEFAULT_USER_AGENT, RequestExt},
        macros::maybe,
    },
};
use scraper::Html;
use std::{fmt::Display, ops::RangeBounds, sync::Arc};

/// A builder for [`Client`] with custom configuration.
///
/// Obtain one via [`ClientBuilder::new()`] and call [`build()`](ClientBuilder::build())
/// when done. For simple cases, [`Client::new()`] or [`Client::with_session()`] are
/// sufficient without a builder.
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
#[derive(Debug)]
pub struct ClientBuilder {
    builder: reqwest::ClientBuilder,
    session: Session,
}

impl Default for ClientBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Creates a new [`ClientBuilder`] with default settings.
    ///
    /// Sets the user agent to `{crate_name}/{crate_version}`.
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
            session: Session::default(),
        }
    }

    /// Sets the session token for authenticated requests.
    ///
    /// This is the `NEO_SES` cookie value. Included in requests where authentication
    /// is required.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::ClientBuilder;
    /// let builder = ClientBuilder::new().with_session("session-token");
    /// ```
    #[inline]
    #[must_use]
    pub fn with_session(self, session: &str) -> Self {
        let mut client = self;
        client.session = Session::new(session);
        client
    }

    /// Overrides the default `User-Agent` header.
    ///
    /// Defaults to `{crate_name}/{crate_version}` if not set.
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
        let client = self;
        let builder = client.builder.user_agent(user_agent);
        Self { builder, ..client }
    }

    /// Consumes this [`ClientBuilder`] and returns a configured [`Client`].
    ///
    /// # Errors
    ///
    /// Returns [`ClientBuilderError`] if the underlying HTTP client could not be built,
    /// e.g. TLS initialization failure or DNS resolver misconfiguration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{ClientBuilder, error::ClientBuilderError, Client};
    /// let client: Client = ClientBuilder::new().build()?;
    /// # Ok::<(), webtoon::platform::webtoons::error::ClientBuilderError>(())
    /// ```
    #[inline]
    pub fn build(self) -> Result<Client, ClientBuilderError> {
        let client = self;
        Ok(Client {
            http: client
                .builder
                .build()
                .map_err(|_err| ClientBuilderError::BuildFailed)?,
            session: client.session,
        })
    }
}

/// An asynchronous client for `webtoons.com`.
///
/// Manages connection pooling internally and is cheap to clone. For custom
/// configuration, use [`ClientBuilder`] via [`Client::builder()`].
///
/// # Examples
///
/// ```
/// # use webtoon::platform::webtoons::Client;
/// let client = Client::new();
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(super) http: reqwest::Client,
    pub(super) session: Session,
}

impl Client {
    /// Creates a new [`Client`] with default settings.
    ///
    /// Sets the user agent to `{crate_name}/{crate_version}`. For custom configuration
    /// or fallible construction, use [`ClientBuilder`] instead.
    ///
    /// # Panics
    ///
    /// Panics if TLS initialization fails or the DNS resolver cannot load the system
    /// configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::Client;
    /// let client = Client::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        #[expect(
            clippy::expect_used,
            reason = "it is documented that this can panic and that `ClientBuilder` should be used instead for a `Result`"
        )]
        ClientBuilder::new().build().expect("Client::new()")
    }

    /// Creates a new [`Client`] with a session token for authenticated requests.
    ///
    /// For custom configuration or fallible construction, use [`ClientBuilder`] instead.
    ///
    /// # Panics
    ///
    /// Panics if TLS initialization fails or the DNS resolver cannot load the system
    /// configuration.
    ///
    /// # Examples
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

    /// Returns a [`ClientBuilder`] for creating a custom-configured [`Client`].
    ///
    /// # Examples
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

impl Client {
    /// Returns the [`Creator`] for the given profile, if any.
    ///
    /// The profile is the last segment of the creator's community page URL, e.g.
    /// `w7m5o` from `https://www.webtoons.com/p/community/en/u/w7m5o`.
    ///
    /// Not all webtoon creators have a community page - this is typically indicated
    /// by a green checkmark next to their name on the webtoon's page.
    ///
    /// Returns `Ok(None)` if no profile exists for the given slug. Returns
    /// [`CreatorError::InvalidCreatorProfile`] if the profile exists but responds
    /// unexpectedly.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, error::{Error, CreatorError}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// match client.creator("w7m5o").await {
    ///     Ok(Some(creator)) => println!("Creator found: {creator:?}"),
    ///     Ok(None) => unreachable!("profile is known to exist"),
    ///     Err(err) => panic!("An error occurred: {err:?}"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn creator(&self, profile: &str) -> Result<Option<Creator>, CreatorError> {
        let client = self;

        let Some(homepage) = creator::homepage(profile, client).await? else {
            return Ok(None);
        };

        Ok(Some(Creator {
            client: client.clone(),
            profile: Some(profile.into()),
            username: homepage.username.clone(),
            homepage: Cache::new(Some(homepage)),
        }))
    }

    /// Searches for webtoons on `webtoons.com` and returns matching [`Item`]s.
    ///
    /// Returns an empty `Vec` for an empty query. Results include both [`Original`](variant@Type::Original)
    /// and [`Canvas`](variant@Type::Canvas) webtoons, and are not ranked or sorted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let results = client.search("Monsters And").await?;
    ///
    /// for webtoon in results {
    ///     println!("Webtoon: {}", webtoon.title());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str) -> Result<Vec<Item>, SearchError> {
        async fn search_by_type(
            client: &Client,
            query: &str,
            r#type: Type,
        ) -> Result<Vec<Item>, SearchError> {
            let mut items = Vec::new();
            let mut cursor: Option<String> = None;

            let subtype = match r#type {
                Type::Original => "WEBTOON",
                Type::Canvas => "CHALLENGE",
            };

            loop {
                let url = match &cursor {
                    Some(c) => format!(
                        "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType={subtype}&nextSize=50&language=ENGLISH&query={query}&cursor={c}"
                    ),
                    None => format!(
                        "https://www.webtoons.com/p/api/community/v1/content/TITLE/GW/search?criteria=KEYWORD_SEARCH&contentSubType={subtype}&nextSize=50&language=ENGLISH&query={query}"
                    ),
                };

                let response = client.http.get(&url).retry().send().await?;

                let json = response.text().await?;

                assume!(
                    !json.is_empty(),
                    "json response from `webtoons.com` search API should never be empty"
                );

                let search = serde_json::from_str::<api::search::RawSearch>(&json)
                    .with_assumption(|| format!("`webtoons.com` {subtype} search API response should be deserializable as `RawSearch`: `{json}`"))?;

                let (data, next) = match r#type {
                    Type::Original => {
                        let list = search.result.webtoon_title_list.assumption(
                            "`webtoonTitleList` field should be present in WEBTOON search result",
                        )?;
                        (list.data, list.pagination.next)
                    }
                    Type::Canvas => {
                        let list = search
                            .result
                            .challenge_title_list
                            .assumption("`challengeTitleList` field should be present in CHALLENGE search result")?;
                        (list.data, list.pagination.next)
                    }
                };

                for item in data {
                    items.push(Item {
                        client: client.clone(),
                        id: item.content_id,
                        r#type,
                        title: item.name,
                        thumbnail: format!(
                            "https://swebtoon-phinf.pstatic.net{}",
                            item.thumbnail.path
                        ),
                        creator: item.extra.writer.nickname,
                    });
                }

                match next {
                    Some(next_cursor) => cursor = Some(next_cursor),
                    None => break,
                }
            }

            maybe!(items.is_empty());

            Ok(items)
        }

        let client = self;

        if query.is_empty() {
            return Ok(Vec::new());
        }

        let mut webtoons = Vec::with_capacity(100);

        webtoons.extend(search_by_type(client, query, Type::Original).await?);
        webtoons.extend(search_by_type(client, query, Type::Canvas).await?);

        maybe!(webtoons.is_empty());

        Ok(webtoons)
    }

    /// Returns all `Original` webtoons from `webtoons.com/en/originals`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{ Client, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let originals = client.originals().await?;
    ///
    /// println!("Found {} originals", originals.len());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn originals(&self) -> Result<Vec<Webtoon>, OriginalsError> {
        let client = self;
        originals::scrape(client).await
    }

    /// Returns `Canvas` webtoons from `webtoons.com/en/canvas`.
    ///
    /// `pages` accepts any `u16` range; an unbounded end is capped at page 100 to
    /// avoid infinite scraping since `webtoons.com` gives no indication when pages
    /// run out.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{ Client, error::Error, canvas::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoons = client
    ///     .canvas(1..=2, Sort::Popularity)
    ///     .await?;
    ///
    /// for webtoon in webtoons {
    ///     println!("Webtoon: {}", webtoon.id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn canvas(
        &self,
        pages: impl RangeBounds<u16> + Send,
        sort: Sort,
    ) -> Result<Vec<Webtoon>, CanvasError> {
        let client = self;
        canvas::scrape(client, pages, sort).await
    }

    /// Returns the [`Webtoon`] with the given `id` and [`Type`], if it exists.
    ///
    /// [`Original`](variant@Type::Original) and [`Canvas`](variant@Type::Canvas) webtoons
    /// have separate id spaces, so the same numeric id can refer to different webtoons
    /// depending on the type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Type, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(95, Type::Original).await?.expect("`95` exists");
    ///
    /// assert_eq!("Tower of God", webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn webtoon(&self, id: u32, r#type: Type) -> Result<Option<Webtoon>, ClientError> {
        let client = self;
        Webtoon::new_with_client(id, r#type, client).await
    }

    /// Returns a [`Webtoon`] constructed from a `webtoons.com` URL.
    ///
    /// The URL must follow the format:
    /// `https://www.webtoons.com/{language}/{scope}/{slug}/list?title_no={id}`
    ///
    /// Unlike [`Client::webtoon()`], this does not make a network request and assumes
    /// the webtoon exists.
    ///
    /// # Examples
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
    /// assert_eq!("Omniscient Reader", webtoon.title().await?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn webtoon_from_url(&self, url: &str) -> Result<Webtoon, InvalidWebtoonUrl> {
        let client = self;
        Webtoon::from_url_with_client(url, client)
    }

    /// Returns [`UserInfo`] for the given session token, if the session is valid.
    ///
    /// Returns `None` for an invalid or unauthenticated session. Useful for resolving
    /// a username or profile from a session token alone.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// // An invalid session returns `None`.
    /// assert!(client.user_info_for_session("session").await?.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn user_info_for_session(
        &self,
        session: &str,
    ) -> Result<Option<UserInfo>, UserInfoError> {
        let client = self;

        let json = client
            .http
            .get("https://www.webtoons.com/en/member/userInfo")
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?
            .text()
            .await?;

        assume!(
            !json.is_empty(),
            "json response from user info API should never be empty"
        );

        let Ok(user) = serde_json::from_str::<UserInfoRaw>(&json) else {
            assumption!(
                "`webtoons.com` `userInfo` response should be deserializable as `UserInfoRaw`: `{json}`"
            )
        };

        if !user.is_logged_in {
            return Ok(None);
        }

        assume!(
            user.username.is_some(),
            "`UserInfoRaw::username` should be `Some` when `is_logged_in` is true: `{json}`"
        );

        let user = match (user.is_canvas_creator, &user.profile) {
            (true, None) => {
                assumption!("`profile` should be `Some` when `is_canvas_creator` is true: `{json}`")
            }
            (false, Some(profile)) => assumption!(
                "`profile` should be `None` when `is_canvas_creator` is false, got `Some({profile})`: `{json}`"
            ),
            (false, None) | (true, Some(_)) => UserInfo::from(user),
        };

        Ok(Some(user))
    }

    /// Returns `true` if this [`Client`] was created with a session token.
    ///
    /// Does not validate the session - only checks that one was provided.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    /// assert!(!client.has_session());
    ///
    /// let client = Client::with_session("session");
    /// assert!(client.has_session());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has_session(&self) -> bool {
        let client = self;
        !client.session.is_empty()
    }

    /// Returns `true` if the current session is valid, `false` if it is not.
    ///
    /// This is a point-in-time check - the session could be invalidated immediately
    /// after this returns. Do not rely on this as a guarantee for subsequent calls.
    ///
    /// If the session is invalid, then it will always remain invalid hereafter.
    ///
    /// # Examples
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
        let client = self;
        match client.session.validate(client).await {
            Ok(_) => Ok(true),
            Err(SessionError::InvalidSession) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

impl Client {
    pub(super) async fn fetch_originals_page(&self, day: &str) -> Result<Html, reqwest::Error> {
        let client = self;
        let url = format!("https://www.webtoons.com/en/originals/{day}");
        let document = client.http.get(&url).retry().send().await?.text().await?;
        let html = Html::parse_document(&document);
        Ok(html)
    }

    pub(super) async fn fetch_canvas_page(
        &self,
        page: u16,
        sort: Sort,
    ) -> Result<Html, reqwest::Error> {
        let client = self;

        let url = format!(
            "https://www.webtoons.com/en/canvas/list?genreTab=ALL&sortOrder={sort}&page={page}"
        );

        let document = client.http.get(&url).retry().send().await?.text().await?;

        let html = Html::parse_document(&document);

        Ok(html)
    }

    pub(super) async fn fetch_creator_page(
        &self,
        profile: &str,
    ) -> Result<Option<Html>, CreatorError> {
        let client = self;

        let url = format!("https://www.webtoons.com/p/community/en/u/{profile}");

        let response = client.http.get(&url).retry().send().await?;

        // MAGIC:
        // `404`: if the page does not exist (the profile does not exist).
        // `400`: if the page is disabled by the creator.
        if matches!(response.status().as_u16(), 404 | 400) {
            return Ok(None);
        }

        let document = response.text().await?;

        let html = Html::parse_document(&document);

        Ok(Some(html))
    }

    // TODO: Need to check if the profile existed as an english profile
    pub(super) async fn fetch_creator_webtoons(
        &self,
        profile: &str,
    ) -> Result<CreatorWebtoons, CreatorWebtoonsError> {
        let client = self;

        let url = format!(
            "https://www.webtoons.com/p/community/api/v1/creator/{profile}/titles?language=ENGLISH"
        );

        let response = client.http.get(url).send().await?;

        let json = response.text().await?;

        let creator_webtoons = serde_json::from_str::<CreatorWebtoons>(&json) //
            .with_assumption(|| {
                format!("`webtoons.com` creator webtoons response should be deserializable as `CreatorWebtoons`: `{json}`")
            })?;

        Ok(creator_webtoons)
    }

    pub(super) async fn fetch_webtoon_homepage(
        &self,
        webtoon: &Webtoon,
        page: Option<u16>,
    ) -> Result<Html, reqwest::Error> {
        let client = self;

        let id = webtoon.id;
        let scope = webtoon.scope.as_slug();
        let slug = &webtoon.slug;

        let url = if let Some(page) = page {
            format!("https://www.webtoons.com/en/{scope}/{slug}/list?title_no={id}&page={page}")
        } else {
            format!("https://www.webtoons.com/en/{scope}/{slug}/list?title_no={id}")
        };

        let response = client.http.get(&url).retry().send().await?.text().await?;

        let html = Html::parse_document(&response);

        Ok(html)
    }

    // TODO:
    // If a session is valid, but does not belong to the Webtoon, then
    // should return `InvalidPermissions`.
    pub(super) async fn fetch_episodes_dashboard(
        &self,
        webtoon: &Webtoon,
        page: u16,
    ) -> Result<Vec<DashboardEpisode>, SessionError> {
        let client = self;

        let session = client.session.validate(client).await?;

        let url = format!(
            "https://www.webtoons.com/*/challenge/dashboardEpisode?titleNo={id}&page={page}",
            id = webtoon.id
        );

        let response = client
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?
            .text()
            .await?;

        let episodes = api::dashboard::episodes::parse(&response)?;

        Ok(episodes)
    }

    pub(super) async fn fetch_series_analytics(
        &self,
        webtoon: &Webtoon,
        page: u16,
    ) -> Result<SeriesAnalytics, SessionError> {
        let client = self;

        let id = webtoon.id;

        let session = client.session.validate(client).await?;

        let url = format!(
            "https://www.webtoons.com/en/api/v1/creators/analytics/episodes?titleNo={id}&page={page}&pageSize=100"
        );

        let json = client
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?
            .text()
            .await?;

        let series_analytics = serde_json::from_str::<SeriesAnalytics>(&json) //
            .with_assumption(|| {
                 format!("`webtoons.com` series analytics response should be deserializable as `SeriesAnalytics`: `{json}`")
            })?;

        Ok(series_analytics)
    }

    #[cfg(feature = "rss")]
    pub(super) async fn rss(&self, webtoon: &Webtoon) -> Result<rss::Channel, WebtoonError> {
        use std::str::FromStr;

        let client = self;

        let id = webtoon.id;
        let scope = match webtoon.scope {
            Scope::Original(genre) => genre.as_slug(),
            Scope::Canvas => "challenge",
        };
        let slug = &webtoon.slug;

        let url = format!("https://www.webtoons.com/en/{scope}/{slug}/rss?title_no={id}");

        let response = client.http.get(url).send().await?.text().await?;

        let rss = rss::Channel::from_str(&response).assumption(
            "rss feed returned from `webtoons.com` should be parseable as a valid RSS channel",
        )?;

        Ok(rss)
    }

    pub(super) async fn fetch_episode_page(
        &self,
        webtoon: &Webtoon,
        episode: u16,
    ) -> Result<Html, EpisodeError> {
        let client = self;

        let id = webtoon.id;
        let scope = webtoon.scope.as_slug();

        // Language isn't needed
        let url = format!(
            "https://www.webtoons.com/*/{scope}/*/*/viewer?title_no={id}&episode_no={episode}"
        );

        let response = client.http.get(&url).retry().send().await?;

        if response.status() == 404 {
            return Err(EpisodeError::NotViewable);
        }

        let document = response.text().await?;

        let html = Html::parse_document(&document);

        Ok(html)
    }

    pub(super) async fn fetch_episodes_likes(
        &self,
        episode: &Episode,
    ) -> Result<RawLikesResponse, LikesError> {
        let client = self;

        let scope = match episode.webtoon.scope {
            Scope::Original(_) => "w",
            Scope::Canvas => "c",
        };
        let webtoon = episode.webtoon.id;
        let episode = episode.number;

        let url = format!(
            "https://www.webtoons.com/api/v1/like/search/counts?serviceId=LINEWEBTOON&contentIds={scope}_{webtoon}_{episode}"
        );

        let response = client.http.get(&url).retry().send().await?.text().await?;

        let raw_likes_response = serde_json::from_str::<RawLikesResponse>(&response)
           .with_assumption(|| format!("`webtoons.com` likes API response should be deserializable as `RawLikesResponse`: `{response}`"))?;

        Ok(raw_likes_response)
    }

    pub(super) async fn fetch_episode_posts(
        &self,
        episode: &Episode,
        cursor: Option<Id>,
        stride: u8,
        pin_representation: PinRepresentation,
    ) -> Result<RawPostResponse, PostsError> {
        let client = self;

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

        let request = match client.session.validate(client).await {
            Ok(session) => client
                .http
                .get(&url)
                .header("Cookie", format!("NEO_SES={session}")),
            Err(SessionError::NoSessionProvided) => client.http.get(&url),
            Err(SessionError::InvalidSession) => return Err(PostsError::InvalidSession),
            Err(SessionError::Internal(err)) => return Err(err.into()),
            Err(SessionError::RequestFailed(err)) => return Err(err.into()),
        };

        let response = request
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await?
            .text()
            .await?;

        let raw_post_response =    serde_json::from_str::<RawPostResponse>(&response)
            .with_assumption(|| format!("`webtoons.com` post API response should be deserializable as `RawPostResponse`: `{response}`"))?;

        Ok(raw_post_response)
    }

    pub(super) async fn check_if_episode_exists(
        &self,
        episode: &Episode,
    ) -> Result<bool, reqwest::Error> {
        let client = self;

        let scope = match episode.webtoon.scope {
            Scope::Original(_) => "w",
            Scope::Canvas => "c",
        };
        let webtoon = episode.webtoon.id;
        let episode = episode.number;

        let url = format!(
            "https://www.webtoons.com/p/api/community/v2/posts?pageId={scope}_{webtoon}_{episode}&pinRepresentation=none&prevSize=0&nextSize=1&cursor=&withCursor=true"
        );

        let response = client
            .http
            .get(&url)
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await?;

        // TODO: This can return false if status is 500, for example, which is wrong.
        Ok(response.status() != 404 && response.status().is_success())
    }

    pub(super) async fn fetch_replies_for_post(
        &self,
        post: &Post,
        cursor: Option<Id>,
        stride: u8,
    ) -> Result<RawPostResponse, PostsError> {
        let client = self;

        let id = post.id;
        let cursor = cursor.map_or_else(String::new, |id| id.to_string());

        let url = format!(
            "https://www.webtoons.com/p/api/community/v2/post/{id}/child-posts?sort=oldest&displayBlindCommentAsService=false&prevSize=0&nextSize={stride}&cursor={cursor}&withCursor=false"
        );

        let request = match client.session.validate(client).await {
            Ok(session) => client
                .http
                .get(&url)
                .header("Cookie", format!("NEO_SES={session}")),
            Err(SessionError::NoSessionProvided) => client.http.get(&url),
            Err(SessionError::InvalidSession) => return Err(PostsError::InvalidSession),
            Err(SessionError::Internal(err)) => return Err(err.into()),
            Err(SessionError::RequestFailed(err)) => return Err(err.into()),
        };

        let response = request
            .header("Service-Ticket-Id", "epicom")
            .retry()
            .send()
            .await?
            .text()
            .await?;

        let raw_post_response = serde_json::from_str::<RawPostResponse>(&response)
            .with_assumption(|| format!("`webtoons.com` post API response should be deserializable as `RawPostResponse`: `{response}`"))?;

        Ok(raw_post_response)
    }

    pub(super) async fn fetch_webtoon_user_info(
        &self,
        session: &ValidSession,
        webtoon: &Webtoon,
    ) -> Result<WebtoonUserInfo, UserInfoError> {
        let client = self;

        let id = webtoon.id;

        let url = match webtoon.r#type() {
            Type::Original => {
                format!("https://www.webtoons.com/getTitleUserInfo?titleNo={id}")
            }
            Type::Canvas => {
                format!("https://www.webtoons.com/canvas/getTitleUserInfo?titleNo={id}")
            }
        };

        let response = client
            .http
            .get(&url)
            .header("Cookie", format!("NEO_SES={session}"))
            .retry()
            .send()
            .await?
            .text()
            .await?;

        let webtoon_user_info =  serde_json::from_str::<WebtoonUserInfo>(&response)
            .with_assumption(|| format!("`webtoons.com` webtoon user info API response should be deserializable as `WebtoonUserInfo`: `{response}`"))?;

        Ok(webtoon_user_info)
    }

    #[cfg(feature = "download")]
    pub(super) async fn download_panel(
        &self,
        url: &reqwest::Url,
    ) -> Result<Vec<u8>, reqwest::Error> {
        let client = self;
        let bytes = client.http.get(url.as_str()).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// A session token that has been verified as valid by `webtoons.com`.
///
/// Obtained via [`Session::validate()`]; cannot be constructed directly. Note that
/// validity is only guaranteed at the time of the check - the session may be
/// invalidated at any point afterward (TOCTOU).
pub(crate) struct ValidSession(Arc<str>);

impl Display for ValidSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let session = &self.0;
        write!(f, "{session}")
    }
}

/// The session token used for authenticated requests.
///
/// Wraps an optional `NEO_SES` cookie value. A `None` inner value means no session
/// was provided; `Some` means one was, but it may still be invalid.
#[derive(Debug, Clone, Default)]
pub(crate) struct Session(Option<Arc<str>>);

impl Session {
    #[inline]
    fn new(session: &str) -> Self {
        Self(Some(Arc::from(session)))
    }

    /// Validates the session against `webtoons.com`, returning a [`ValidSession`] if successful.
    ///
    /// Returns [`SessionError::NoSessionProvided`] if no session was set, or
    /// [`SessionError::InvalidSession`] if the session is rejected by the platform.
    pub async fn validate(&self, client: &Client) -> Result<ValidSession, SessionError> {
        let session = &self.0;

        let Some(session) = &session else {
            return Err(SessionError::NoSessionProvided);
        };

        if client.user_info_for_session(session).await?.is_none() {
            return Err(SessionError::InvalidSession);
        }

        Ok(ValidSession(session.clone()))
    }

    /// Returns `true` if no session was provided or the session string is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        let session = &self.0;
        session.as_ref().is_none_or(|session| session.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_should_be_empty() {
        let session = Session::default();
        assert!(session.is_empty());
        let session = Session::new("");
        assert!(session.is_empty());
    }

    #[test]
    fn session_should_not_be_empty() {
        let session = Session::new("session");
        assert!(!session.is_empty());
    }
}
