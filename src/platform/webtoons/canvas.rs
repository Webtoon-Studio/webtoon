//! Module represening the canvas story list.
//!
//! ## Example
//! ```rust,no_run
//! # use webtoon::platform::webtoons::{ Client, Language, errors::Error, canvas::Sort};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Error> {
//! # let client = Client::new();
//! let webtoons = client
//!     .canvas(Language::En, 1..=3, Sort::Popularity)
//!     .await?;
//!
//! for webtoon in webtoons {
//!     println!("Webtoon: {}", webtoon.title().await?);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Context, Result};
use scraper::{Html, Selector};
use std::{fmt::Display, ops::RangeBounds, time::Duration};

use super::{
    errors::{CanvasError, ClientError},
    Client, Language, Webtoon,
};

pub(super) async fn scrape(
    client: &Client,
    language: Language,
    pages: impl RangeBounds<u16> + Send,
    sort: Sort,
) -> Result<Vec<Webtoon>, CanvasError> {
    // NOTE: currently all languages are the same
    let selector = Selector::parse("div.challenge_lst>ul>li>a") //
        .expect("`div.challenge_lst>ul>li>a` should be a valid selector");

    let start = match pages.start_bound() {
        std::ops::Bound::Included(&n) => n.max(1),
        std::ops::Bound::Excluded(&n) => n + 1,
        std::ops::Bound::Unbounded => 1,
    };

    let end = match pages.end_bound() {
        std::ops::Bound::Included(&n) => n + 1,
        std::ops::Bound::Excluded(&n) => n,
        std::ops::Bound::Unbounded => 100,
    };

    if start > end {
        return Err(CanvasError::Unexpected(anyhow!(
            "range start was greater than range end",
        )));
    }

    let mut webtoons = Vec::with_capacity(usize::from(end - start + 1) * 20);

    for page in start..end {
        let response = match client.get_canvas_page(language, page, sort).await {
            Ok(response) => response,
            Err(ClientError::RateLimitExceeded(retry_after)) => {
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                client.get_canvas_page(language, page, sort).await?
            }
            Err(err) => return Err(CanvasError::ClientError(err)),
        };

        let document = response.text().await?;

        let html = Html::parse_document(&document);

        for card in html.select(&selector) {
            let href = card
                .attr("href")
                .context("`href` is missing, `a` tag should always have one")?;

            webtoons.push(Webtoon::from_url_with_client(href, client)?);
        }
    }

    Ok(webtoons)
}

/// Represents sorting options when scraping `www.webtoons.com/*/canvas/list`
#[derive(Debug, Clone, Copy)]
pub enum Sort {
    /// Sort by views.
    Popularity,
    /// Sort by likes.
    Likes,
    /// Sort by newest upload.
    Date,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sort = match self {
            Self::Popularity => "READ_COUNT",
            Self::Likes => "LIKEIT",
            Self::Date => "UPDATE",
        };

        write!(f, "{sort}")
    }
}
