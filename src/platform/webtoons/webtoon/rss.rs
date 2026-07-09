//! Module representing a webtoons RSS feed.

use assumptions::{Assume, Assumption, assume};
use chrono::{DateTime, NaiveDateTime, Utc};
use url::Url;

use std::debug_assert as ensure;
use std::str::FromStr;

use crate::{
    platform::webtoons::{creator::Creator, error::WebtoonError, webtoon::episode::Published},
    stdx::cache::Cache,
};

use super::{
    Webtoon,
    episode::{Episode, PublishedStatus},
};

/// RSS feed data for a [`Webtoon`].
///
/// Not a spec-compliant RSS representation; shaped around what `webtoons.com` exposes.
#[derive(Debug)]
pub struct Rss {
    pub(super) url: String,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) thumbnail: String,
    pub(super) creators: Vec<Creator>,
    pub(super) episodes: Vec<Episode>,
}

impl Rss {
    /// Returns the Webtoon page URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the name of the Webtoon.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the summary of the Webtoon.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the thumbnail of the Webtoon.
    #[must_use]
    pub fn thumbnail(&self) -> &str {
        &self.thumbnail
    }

    /// Returns the creators of the Webtoon.
    #[must_use]
    pub fn creators(&self) -> &[Creator] {
        &self.creators
    }

    /// Returns the most recent episodes of the Webtoon.
    #[must_use]
    pub fn episodes(&self) -> &[Episode] {
        &self.episodes
    }
}

pub(super) async fn feed(webtoon: &Webtoon) -> Result<Rss, WebtoonError> {
    let channel = webtoon.client.rss(webtoon).await?;

    let mut episodes = Vec::new();

    for item in &channel.items {
        let datetime =
            published(item.pub_date().assumption(
                "`webtoons.com` RSS feed item should always have a `pubDate` element",
            )?)?;

        let number = episode(
            item.link
                .as_ref()
                .assumption("`webtoons.com` RSS feed item should always have a `link` element")?,
        )?;

        let title = item
            .title
            .as_ref()
            .assumption("`webtoons.com` RSS feed item should always have a `title` element")?
            .clone();

        let published = Published::from(datetime);

        assume!(
            published.year() >= 2014,
            "`webtoons.com` episode publish year should be 2014 or later, got: {}",
            published.year()
        );

        episodes.push(Episode {
            webtoon: webtoon.clone(),
            number,
            title: Cache::new(title),
            published: Some(published),
            // RSS can only be generated for public and free(not behind ad or fast-pass) episodes.
            published_status: Some(PublishedStatus::Published),

            length: Cache::empty(),
            thumbnail: Cache::empty(),
            note: Cache::empty(),
            panels: Cache::empty(),
            views: None,
            ad_status: None,
            top_comments: Cache::empty(),
        });
    }

    Ok(Rss {
        title: channel.title.clone(),
        url: channel.link.clone(),
        thumbnail: channel
            .image()
            .assumption("`webtoons.com` Webtoon RSS feed should have an `image` element")?
            .url
            .clone(),
        creators: webtoon.creators().await?,
        summary: channel.description,
        episodes,
    })
}

fn published(date: &str) -> Result<DateTime<Utc>, Assumption> {
    assume!(
        date.ends_with("GMT"),
        "`webtoons.com` RSS feed `pubDate` should end with `GMT`, got: `{date}`"
    );

    let date = date
        .split_once(',')
        .map(|(_, date)| date.trim_end_matches("GMT").trim())
        .with_assumption(|| format!("`webtoons.com` RSS feed `pubDate` should contain `,` separating day of week from date, got: `{date}`"))?;

    assume!(
        date.chars()
            .next()
            .is_some_and(|char| char.is_ascii_digit()),
        "`webtoons.com` RSS feed `pubDate` should start with a digit after the day of week, got: `{date}`"
    );

    ensure!(
        !date.ends_with("GMT"),
        "`pubDate` should not end with `GMT` after trimming"
    );

    let date = NaiveDateTime::parse_from_str(date, "%d %b %Y %T").with_assumption(|| {
        format!("`webtoons.com` RSS feed `pubDate` should be parseable with format `%d %b %Y %T`, got: `{date}`")
    })?;

    Ok(date.and_utc())
}

fn episode(url: &str) -> Result<u16, Assumption> {
    let url = Url::parse(url).with_assumption(|| {
        format!("`webtoons.com` RSS feed episode url should be a valid url, got: `{url}`")
    })?;

    let value = url
        .query_pairs()
        .find(|(key, _)| key == "episode_no")
        .map(|(_, v)| v)
        .with_assumption(|| {
            format!("`webtoons.com` RSS feed episode url should have an `episode_no` query parameter, got: `{url}`")
        })?;

    let number = u16::from_str(&value).with_assumption(|| {
        format!("`episode_no` query parameter should be parseable as a `u16`, got: `{value}`")
    })?;

    Ok(number)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn should_parse_en_rss_date() {
        let date = published("Tuesday, 10 Sep 2024 16:40:23 GMT").unwrap();
        assert_eq!(1725986423, date.timestamp());
    }
}
