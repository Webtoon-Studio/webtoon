use assumptions::{Assume, Assumption};
use chrono::DateTime;
use serde::Deserialize;

use crate::{
    platform::webtoons::{Webtoon, webtoon::episode::Published},
    stdx::cache::Cache,
};

#[derive(Deserialize)]
#[expect(unused)]
pub struct SeriesAnalytics {
    #[serde(rename = "currentPage")]
    pub current_page: u16,
    pub episodes: Vec<Episode>,
    #[serde(rename = "titleName")]
    pub title_name: String,
    #[serde(rename = "totalCount")]
    pub total_count: u16,
    #[serde(rename = "totalPages")]
    pub total_pages: u16,
}

#[derive(Deserialize, Clone)]
pub struct Episode {
    #[serde(rename = "episodeNo")]
    pub number: u16,
    #[serde(rename = "episodeTitle")]
    pub title: String,
    pub comments: Option<u32>,
    #[serde(rename = "pageViews")]
    pub views: Option<u32>,
    #[serde(rename = "publishedDate")]
    pub published: i64,
    #[serde(default)]
    #[serde(rename = "superLikes")]
    pub super_likes: Option<u32>,
}

impl TryFrom<(&Webtoon, Episode)> for crate::platform::webtoons::webtoon::episode::Episode {
    type Error = Assumption;

    fn try_from((webtoon, episode): (&Webtoon, Episode)) -> Result<Self, Self::Error> {
        Ok(Self {
            webtoon: webtoon.clone(),
            number: episode.number,
            title: Cache::new(episode.title),
            published: Some(Published::from(
                DateTime::from_timestamp_millis(episode.published) //
                    .assumption(
                        "timestamps returned by `webtoons.com` should be valid Unix timestamps",
                    )?,
            )),
            views: Some(episode.views.unwrap_or_default()),

            length: Cache::empty(),
            thumbnail: Cache::empty(),
            note: Cache::empty(),
            ad_status: None,
            published_status: None,
            panels: Cache::empty(),
            top_comments: Cache::empty(),
        })
    }
}
