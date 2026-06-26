//! Module representing a webtoons RSS feed.

use chrono::{DateTime, NaiveDateTime, Utc};
use std::str::FromStr;
use url::Url;

use crate::{
    platform::webtoons::{
        creator::Creator,
        error::{RssError, WebtoonError},
        webtoon::episode::Published,
    },
    stdx::{
        cache::Cache,
        error::{Assume, Assumption, assume},
    },
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

pub(super) async fn feed(webtoon: &Webtoon) -> Result<Rss, RssError> {
    let channel = webtoon.client.rss(webtoon).await?;

    let mut episodes = Vec::new();

    for item in &channel.items {
        let datetime = published(
            item.pub_date()
                .assumption("publish date should always be present in `webtoons.com` rss feed, as this feed only shows published episodes")?,
        )?;

        let number = episode(
            item.link
                .as_ref()
                .assumption("rss `link` tag should always be filled, as this represents the link to the `webtoons.com` episode")?,
        )?;

        let title = item
            .title
            .as_ref()
            .assumption("rss feed for `webtoons.com` should always have a Webtoon tile")?
            .clone();

        let published = Published::from(datetime);

        assume!(
            published.year() >= 2014,
            "`webtoons.com` only started in 2014"
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
            .assumption("`webtoons.com` Webtoon rss feed should should have an `image`, represening the thumbnail of the Webtoon")?
            .url
            .clone(),
        creators: webtoon.creators().await.map_err(|err| match err {
                WebtoonError::Internal(err) => RssError::from(err),
                WebtoonError::RequestFailed(err) => RssError::from(err),
            })?,
        summary: channel.description,
        episodes,
    })
}

fn published(date: &str) -> Result<DateTime<Utc>, Assumption> {
    assume!(
        date.ends_with("GMT"),
        "all known rss date formats end with `GMT`"
    );

    let date = date
        .split_once(',')
        .map(|(_, date)| date.trim_end_matches("GMT").trim())
        .with_assumption(|| format!("incoming `date` should always be able to split once on `,`, as all formats should begin with `day of week,`, so should always be `Some`, but got `{date}`"))?;

    assume!(
        date.chars()
            .next()
            .is_some_and(|char| char.is_ascii_digit()),
        "`date` should start with a digit after splitting on `,`"
    );

    assume!(
        !date.ends_with("GMT"),
        "`date` should not end with `GMT` after trimming"
    );

    let date = NaiveDateTime::parse_from_str(date, "%d %b %Y %T").with_assumption(|| {
        format!(
            "`webtoons.com` Webtoon RSS feed `pubDate` should always be a known format `{date}`"
        )
    })?;

    Ok(date.and_utc())
}

fn episode(url: &str) -> Result<u16, Assumption> {
    let url = Url::parse(url).with_assumption(|| {
        format!("urls returned from `webtoons.com` rss feed should always be valid `{url}`")
    })?;

    let value = url
        .query_pairs()
        .find(|(key, _)| key == "episode_no")
        .map(|(_, v)| v)
        .with_assumption(|| {
            format!(
                "`webtoons.com` Webtoon rss url should always have an `episode_no` query: `{url}`"
            )
        })?;

    let number =   u16::from_str(&value)
        .with_assumption(|| format!("`episode_no` should always have a number parsable to a `u16`, as no `webtoons.com` Webtoon should have more than `u16::MAX` episodes `{value}`"))?;

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
