use chrono::DateTime;
use parking_lot::RwLock;
use serde::Deserialize;
use thiserror::Error;

use crate::platform::webtoons::{
    client::api::dashboard::episodes::DashboardEpisode,
    errors::EpisodeError,
    webtoon::{
        Webtoon,
        episode::{self, AdStatus, Episode},
    },
};
use std::{collections::HashSet, str::FromStr, sync::Arc, time::Duration};

pub async fn scrape(webtoon: &Webtoon) -> Result<Vec<Episode>, EpisodeError> {
    // WARN: There must not be any mutating of episodes while in the HashSet, only inserts.
    #[allow(clippy::mutable_key_type)]
    let mut episodes: HashSet<Episode> = HashSet::new();

    let response = webtoon
        .client
        .get_episodes_dashboard(webtoon, 1)
        .await?
        .text()
        .await?;

    let pages = calculate_max_pages(&response)?;

    let dashboard_episodes =
        crate::platform::webtoons::client::api::dashboard::episodes::DashboardEpisode::parse(
            &response,
        )?;

    for episode in dashboard_episodes {
        episodes.insert(Episode {
            webtoon: webtoon.clone(),
            number: episode.metadata.number,
            season: Arc::new(RwLock::new(episode::season(&episode.metadata.title))),
            title: Arc::new(RwLock::new(Some(episode.metadata.title))),
            published: episode.published.map(|timestamp| {
                DateTime::from_timestamp_millis(timestamp)
                    .expect("webtoons should be using proper timestamps")
            }),
            length: Arc::new(RwLock::new(None)),
            thumbnail: Arc::new(RwLock::new(None)),
            note: Arc::new(RwLock::new(None)),
            panels: Arc::new(RwLock::new(None)),
            views: Some(episode.metadata.views),
            ad_status: Some(episode.dashboard_status.ad_status()),
            published_status: Some(episode.dashboard_status.into()),
        });
    }

    for page in 2..=pages {
        let response = webtoon
            .client
            .get_episodes_dashboard(webtoon, page)
            .await?
            .text()
            .await?;

        let dashboard_episodes = DashboardEpisode::parse(&response)?;

        for episode in dashboard_episodes {
            episodes.insert(Episode {
                webtoon: webtoon.clone(),
                number: episode.metadata.number,
                season: Arc::new(RwLock::new(episode::season(&episode.metadata.title))),
                title: Arc::new(RwLock::new(Some(episode.metadata.title))),
                published: episode.published.map(|timestamp| {
                    DateTime::from_timestamp_millis(timestamp)
                        .expect("webtoons should be using proper timestamps")
                }),
                length: Arc::new(RwLock::new(None)),
                thumbnail: Arc::new(RwLock::new(None)),
                note: Arc::new(RwLock::new(None)),
                panels: Arc::new(RwLock::new(None)),
                views: Some(episode.metadata.views),
                ad_status: Some(episode.dashboard_status.ad_status()),
                published_status: Some(episode.dashboard_status.into()),
            });
        }

        // Sleep for one second to prevent getting a 429 response code for going between the pages to quickly.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let mut episodes: Vec<Episode> = episodes.into_iter().collect();

    episodes.sort_unstable_by_key(Episode::number);

    Ok(episodes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum DashboardStatus {
    Published,
    Draft,
    Approved,
    Removed,
    Ready,
    AdOn,
    AdOff,
    InReview,
    Disapproved,
    DisapprovedAuto,
}

impl DashboardStatus {
    #[allow(dead_code)]
    pub fn is_published(self) -> bool {
        matches!(self, Self::Published | Self::AdOn | Self::AdOff)
    }

    pub fn ad_status(self) -> AdStatus {
        match self {
            Self::Published
            | Self::Draft
            | Self::Ready
            | Self::Approved
            | Self::Removed
            | Self::InReview
            | Self::Disapproved
            | Self::DisapprovedAuto => AdStatus::Never,
            Self::AdOn => AdStatus::Yes,
            Self::AdOff => AdStatus::No,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "failed to parse `{0}` into a `DashboardStatus` expected one of PUBLISHED, READY, DRAFT, IN_REVIEW, APPROVED, REMOVED, AD_ON, or AD_OFF"
)]
pub struct DashboardStatusParseError(String);

impl FromStr for DashboardStatus {
    type Err = DashboardStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PUBLISHED" => Ok(Self::Published),
            "DRAFT" => Ok(Self::Draft),
            "READY" => Ok(Self::Ready),
            "AD_ON" => Ok(Self::AdOn),
            "AD_OFF" => Ok(Self::AdOff),
            "REMOVED" => Ok(Self::Removed),
            "APPROVED" => Ok(Self::Approved),
            "IN_REVIEW" => Ok(Self::InReview),
            "DISAPPROVED" => Ok(Self::Disapproved),
            "DISAPPROVED_AUTO" => Ok(Self::DisapprovedAuto),
            unknown => Err(DashboardStatusParseError(unknown.to_string())),
        }
    }
}

impl TryFrom<String> for DashboardStatus {
    type Error = DashboardStatusParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

// TODO: Use the `stdx::math` trait like in other places with episode per page calculation.
fn calculate_max_pages(html: &str) -> Result<u16, EpisodeError> {
    let episodes = DashboardEpisode::parse(html)?;

    if episodes.is_empty() {
        return Ok(0);
    }

    let latest = episodes[0].metadata.number;

    // 10 per page. Gets within -1 of the actual page count if there is overflow
    let min = latest / 10;

    // Checks for overflow chapters that would make an extra page
    // If there is any excess it will at most be one extra page, and so if true, the value becomes `1`
    // later added to the page count from before
    let overflow = u16::from((latest % 10) != 0);

    let pages = min + overflow;

    Ok(pages)
}
