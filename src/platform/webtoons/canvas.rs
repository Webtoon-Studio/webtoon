//! Module representing the canvas story list as `www.webtoons.com/*/canvas/list`.
//!
//! # Example
//!
//! ```
//! # use webtoon::platform::webtoons::{ Client, Language, errors::Error, canvas::Sort};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Error> {
//! let client = Client::new();
//!
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

use scraper::Selector;
use std::{fmt::Display, ops::RangeBounds};

use crate::{
    platform::webtoons::error::CanvasError,
    stdx::error::{Invariant, invariant},
};

use super::{Client, Language, Webtoon};

pub(super) async fn scrape(
    client: &Client,
    language: Language,
    pages: impl RangeBounds<u16>,
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

    let mut webtoons = Vec::with_capacity(usize::from(end - start + 1) * 20);

    for page in start..end {
        let html = client.get_canvas_page(language, page, sort).await?;

        for card in html.select(&selector) {
            let href = card.attr("href").invariant(
                "`href` attribute is missing on `webtoon.com` `Canvas` page, `a` tag should always have one",
            )?;

            let webtoon = match Webtoon::from_url_with_client(href, client) {
                Ok(webtoon) => webtoon,
                Err(err) => invariant!(
                    "url's found on `webtoons.com` Canvas page should be valid urls that can be turned into a `Webtoon`: {err}"
                ),
            };

            webtoons.push(webtoon);
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
