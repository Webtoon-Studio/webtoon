//! Canvas story list at `https://www.webtoons.com/en/canvas/list`.

use super::{Client, Webtoon};
use crate::{
    platform::webtoons::error::CanvasError,
    stdx::error::{Assume, assume},
};
use scraper::Selector;
use std::{fmt::Display, ops::RangeBounds};

pub(super) async fn scrape(
    client: &Client,
    pages: impl RangeBounds<u16>,
    sort: Sort,
) -> Result<Vec<Webtoon>, CanvasError> {
    let selector = Selector::parse("div.challenge_lst>ul>li>a") //
        .assumption("`div.challenge_lst>ul>li>a` should be a valid selector")?;

    let start = match pages.start_bound() {
        // 1..=100, 1..100, 1.. -> start at page 1
        std::ops::Bound::Included(&n) => n.max(1),
        // (0..=100), (0..100) -> start at page 1 (webtoons.com pages are 1-indexed)
        std::ops::Bound::Excluded(&n) => (n + 1).max(1),
        // ..=100, ..100, .. -> start at page 1
        std::ops::Bound::Unbounded => 1,
    };

    let end = match pages.end_bound() {
        // 1..=100 -> end at page 100 (inclusive)
        std::ops::Bound::Included(&n) => n + 1,
        // 1..100 -> end at page 100 (exclusive, so n is already the correct bound)
        std::ops::Bound::Excluded(&n) => n,
        // 1.., .. -> cap at page 100 to avoid unbounded scraping
        std::ops::Bound::Unbounded => 100,
    };

    // MAGIC: `20`: webtoons per page.
    let mut webtoons = Vec::with_capacity(usize::from(end - start + 1) * 20);

    for page in start..end {
        let html = client.fetch_canvas_page(page, sort).await?;

        for card in html.select(&selector) {
            let href = card.attr("href").assumption(
                "`href` attribute is missing on `webtoon.com` `Canvas` page, `a` tag should always have one",
            )?;

            let webtoon =  Webtoon::from_url_with_client(href, client)
                .with_assumption(|| format!("url's found on `webtoons.com` Canvas page should be valid urls that can be turned into a `Webtoon` `{href}`"))?;

            webtoons.push(webtoon);
        }
    }

    assume!(
        !webtoons.is_empty(),
        "`webtoons.com` `Canvas` page has 20 webtoon cards per page, so should never be empty"
    );
    assume!(
        webtoons.len() % 20 == 0,
        "`webtoons.com` `Canvas` page has 20 webtoon cards per page, so should `webtoons % 20 == 0`"
    );

    Ok(webtoons)
}

/// Sorting options for the `Canvas` story list.
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
        // NOTE:
        // This has already had an instance where the text representation has
        // changed, `READ_COUNT` -> `MANA`, but there isn't a nice way to test
        // this. It must be kept in kind this can change!
        let sort = match self {
            Self::Popularity => "MANA",
            Self::Likes => "LIKEIT",
            Self::Date => "UPDATE",
        };

        write!(f, "{sort}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_popularity_displays_as_mana() {
        assert_eq!(Sort::Popularity.to_string(), "MANA");
    }

    #[test]
    fn sort_likes_displays_as_likeit() {
        assert_eq!(Sort::Likes.to_string(), "LIKEIT");
    }

    #[test]
    fn sort_date_displays_as_update() {
        assert_eq!(Sort::Date.to_string(), "UPDATE");
    }
}
