//! Module representing a webtoons RSS feed.

use chrono::{DateTime, NaiveDateTime, Utc};
use std::str::FromStr;
use url::Url;

use crate::{
    platform::webtoons::{Language, creator::Creator},
    stdx::{
        cache::Cache,
        error::{Assume, Assumption, assumption},
    },
};

use super::{
    Webtoon, WebtoonError,
    episode::{Episode, PublishedStatus},
};

/// Represents the RSS data from the webtoons.com RSS feed for the Webtoon.
///
/// This is not a spec-compliant representation, but rather one that would make sense from a `webtoons.com` perspective.
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
    let channel = webtoon.client.get_rss_for_webtoon(webtoon).await?;

    let mut episodes = Vec::new();

    for item in &channel.items {
        let published = published(
            item.pub_date()
                .assumption("publish date should always be present in `webtoons.com` rss feed, as this feed only shows published episodes")?,
            webtoon.language(),
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

        episodes.push(Episode {
            webtoon: webtoon.clone(),
            number,
            title: Cache::new(title),
            published: Some(published),
            // RSS can only be generated for public and free(not behind ad or fast-pass) episodes.
            published_status: Some(PublishedStatus::Published),

            season: Cache::empty(),
            length: Cache::empty(),
            thumbnail: Cache::empty(),
            note: Cache::empty(),
            panels: Cache::empty(),
            views: None,
            ad_status: None,
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
        creators: webtoon.creators().await?,
        summary: channel.description,
        episodes,
    })
}

fn published(date: &str, language: Language) -> Result<DateTime<Utc>, Assumption> {
    assumption!(
        date.ends_with("GMT"),
        "all known rss date formats end with `GMT`"
    );

    #[allow(clippy::match_same_arms)]
    let fmt = match language {
        // Tuesday, 10 Sep 2024 16:40:23 GMT
        Language::En => "%d %b %Y %T",
        // 星期二, 17 9月 2024 13:01:22 GMT
        Language::Zh => todo!(), //"%e %m月 %Y %T", NOTE: This works as far as I know but will just focus on English for now.
        // วันอังคาร, 17 ก.ย. 2024 13:04:59 GMT
        Language::Th => todo!(), // "%d %b %Y %T",
        // Selasa, 17 Sep 2024 15:03:59 GMT
        Language::Id => todo!(), //"%d %b %Y %T",
        // miércoles, 18 sept. 2024 01:01:48 GMT
        Language::Es => todo!(), // "%d %b. %Y %T",
        // mercredi, 18 sept. 2024 14:01:48 GMT
        Language::Fr => todo!(), //"%d %b. %Y %T",
        // Mittwoch, 18 Sep. 2024 14:01:20 GMT
        Language::De => todo!(), //"%d %b. %Y %T",
    };

    let Some(date) = date
        .split_once(',')
        .map(|(_, date)| date.trim_end_matches("GMT"))
        .map(|date| date.trim())
    else {
        assumption!(
            "incoming `date` should always be able to split once on `,`, as all formats should begin with `day of week,`, so should always be `Some`, but got: `{date}`"
        );
    };

    assumption!(
        date.chars()
            .next()
            .is_some_and(|char| char.is_ascii_digit()),
        "`date` should start with a digit after splitting on `,`"
    );

    assumption!(
        !date.ends_with("GMT"),
        "`date` should not end with `GMT` after trimming"
    );

    match NaiveDateTime::parse_from_str(date, fmt) {
        Ok(date) => Ok(date.and_utc()),
        Err(err) => {
            assumption!(
                "`webtoons.com` Webtoon RSS feed `pubDate` should always be a known format: {err}\n\n`{fmt}`:`{date}`"
            )
        }
    }
}

fn episode(url: &str) -> Result<u16, Assumption> {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(err) => assumption!(
            "urls returned from `webtoons.com` rss feed should always be valid: {err}\n\n`{url}`"
        ),
    };

    let Some((key, value)) = url.query_pairs().nth(1) else {
        assumption!(
            "`webtoons.com` Webtoon rss url should always 2 queries, one for the Webtoon id, `title_no`, and one for the episode number, `episode_no`: `{url}`"
        )
    };

    assumption!(
        key == "episode_no",
        "second url query in `webtoons.com` Webtoon rss feed url was not `episode_no`: {url}"
    );

    match u16::from_str(&value) {
        Ok(episode) => Ok(episode),
        Err(err) => assumption!(
            "`episode_no` should always have a number parsable to a `u16`, as no `webtoons.com` Webtoon should have more than `u16::MAX` episodes: {err}\n\n`{value}`"
        ),
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn should_parse_en_rss_date() {
        let date = published("Tuesday, 10 Sep 2024 16:40:23 GMT", Language::En).unwrap();
        assert_eq!(1725986423, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_zh_rss_date() {
        let date = published("星期二, 17 9月 2024 13:01:22 GMT", Language::Zh).unwrap();
        assert_eq!(1726578082, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_th_rss_date() {
        let date = published("วันอังคาร, 17 ก.ย. 2024 13:04:59 GMT", Language::Th).unwrap();
        assert_eq!(0, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_id_rss_date() {
        let date = published("Selasa, 17 Sep 2024 15:03:59 GMT", Language::Id).unwrap();
        assert_eq!(1726585439, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_es_rss_date() {
        let date = published("miércoles, 18 sept. 2024 01:01:48 GMT", Language::Es).unwrap();
        assert_eq!(0, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_fr_rss_date() {
        let date = published("mercredi, 18 sept. 2024 14:01:48 GMT", Language::Fr).unwrap();
        assert_eq!(0, date.timestamp());
    }

    #[test]
    #[ignore = "todo"]
    fn should_parse_de_rss_date() {
        let date = published("Mittwoch, 18 Sep. 2024 14:01:20 GMT", Language::De).unwrap();
        assert_eq!(1726668080, date.timestamp());
    }
}
