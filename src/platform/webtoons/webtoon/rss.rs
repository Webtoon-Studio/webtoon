//! Module representing a webtoons rss feed.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::{str::FromStr, sync::Arc};
use url::Url;

use crate::platform::webtoons::{Language, creator::Creator};

use super::{
    Webtoon, WebtoonError,
    episode::{Episode, PublishedStatus},
};

/// Represents the RSS data from the webtoons.com rss feed for the webtoon
///
/// This is not a spec-compliant representation, but rather one that would make sense from a webtoon.com perspective.
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
    /// Returns the webtoon page URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the name of the webtoon.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the summary of the webtoon.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the thumbnail of the webtoon.
    #[must_use]
    pub fn thumbnail(&self) -> &str {
        &self.thumbnail
    }

    /// Returns the creators of the webtoon.
    #[must_use]
    pub fn creators(&self) -> &[Creator] {
        &self.creators
    }

    /// Returns the most recent episodes of the webtoon.
    #[must_use]
    pub fn episodes(&self) -> &[Episode] {
        &self.episodes
    }
}

pub(super) async fn feed(webtoon: &Webtoon) -> Result<Rss, WebtoonError> {
    let response = webtoon
        .client
        .get_rss_for_webtoon(webtoon)
        .await?
        .text()
        .await?;

    let channel = rss::Channel::from_str(&response) //
        .map_err(|err| WebtoonError::Unexpected(err.into()))?;

    let mut episodes = Vec::new();

    for item in &channel.items {
        let published = published(
            item.pub_date()
                .expect("publish date should always be in the rss feed"),
            webtoon.language(),
        );

        let number = episode(
            item.link
                .as_ref()
                .expect("rss `link` tag should always be filled"),
        );

        let title = item
            .title
            .as_ref()
            .expect("RSS should always have a tile")
            .clone();

        episodes.push(Episode {
            webtoon: webtoon.clone(),
            number,
            season: Arc::new(RwLock::new(None)),
            title: Arc::new(RwLock::new(Some(title))),
            published: Some(published),
            length: Arc::new(RwLock::new(None)),
            thumbnail: Arc::new(RwLock::new(None)),
            note: Arc::new(RwLock::new(None)),
            panels: Arc::new(RwLock::new(None)),
            views: None,
            ad_status: None,
            // RSS can only be generated for public and free(not behind ad or fast-pass) episodes.
            published_status: Some(PublishedStatus::Published),
        });
    }

    Ok(Rss {
        title: channel.title.clone(),
        url: channel.link.clone(),
        thumbnail: channel
            .image()
            .expect("webtoon rss should have image")
            .url
            .clone(),
        creators: webtoon.creators().await?,
        summary: channel.description,
        episodes,
    })
}

fn published(date: &str, language: Language) -> DateTime<Utc> {
    match language {
        Language::En => {
            // EX: Tuesday, 10 Sep 2024 16:40:23 GMT
            let date = date.replace("GMT", "+0000");

            let date = DateTime::parse_from_str(&date, "%A, %d %b %Y %T %z")
                .expect("RSS feed `pubDate` should always be the same format");

            DateTime::<Utc>::from(date)
        }
        Language::Zh => {
            // EX: 星期二, 17 9月 2024 13:01:22 GMT
            let mut date = date
                .split_once(' ')
                .map(|(_, date)| date.trim_end_matches("GMT"))
                .expect("chinese date should have space in it")
                .to_string();

            date.push_str("+0000");

            DateTime::parse_from_str(&date, "%d年 %m月 %Y %T %z")
                .expect("chinese rss date should have pattern `17 9月 2024 13:01:22 +0000`")
                .into()
        }
        // วันอังคาร, 17 ก.ย. 2024 13:04:59 GMT
        Language::Th => todo!(),
        // Selasa, 17 Sep 2024 15:03:59 GMT
        Language::Id => todo!(),
        // miércoles, 18 sept. 2024 01:01:48 GMT
        Language::Es => todo!(),
        // mercredi, 18 sept. 2024 14:01:48 GMT
        Language::Fr => todo!(),
        // Mittwoch, 18 Sep. 2024 14:01:20 GMT
        Language::De => todo!(),
    }
}

fn episode(url: &str) -> u16 {
    let url = Url::parse(url).expect("RSS generated url should always be valid");

    let (key, value) = url
        .query_pairs()
        .nth(1)
        .expect("url should always 2 queries");

    assert_eq!(
        key, "episode_no",
        "second url query in rss url was not `episode_no`"
    );

    u16::from_str(&value).expect("`episode_no` should always have a number parsable to a u16")
}
