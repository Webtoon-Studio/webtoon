use chrono::DateTime;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    platform::webtoons::{
        error::EpisodeError,
        webtoon::{
            Webtoon,
            episode::{self, AdStatus, Episode},
        },
    },
    stdx::{cache::Cache, error::invariant, math::MathExt},
};
use std::{collections::HashSet, str::FromStr, time::Duration};

pub async fn scrape(webtoon: &Webtoon) -> Result<Vec<Episode>, EpisodeError> {
    #[expect(
        clippy::mutable_key_type,
        reason = "`Episode` has interior mutability, but `Hash` is only on the `number`"
    )]
    let mut episodes: HashSet<Episode> = HashSet::new();

    let dashboard_episodes = webtoon.client.get_episodes_dashboard(webtoon, 1).await?;

    // This gets the highest numerical episode number and calculates what page it
    // would be in, given max episodes per page value. This gives us how many
    // pages we need to go through.
    let pages = dashboard_episodes[0].metadata.number.in_bucket_of(10);

    for episode in dashboard_episodes {
        let published = match episode
            .published
            .map(|timestamp| DateTime::from_timestamp_millis(timestamp))
        {
            Some(Some(published)) => Some(published),
            Some(None) => invariant!(
                "`webtoons.com` should always return a valid unix millisecond timestamp, got: {:?}",
                episode.published
            ),
            None => None,
        };

        episodes.insert(Episode {
            webtoon: webtoon.clone(),
            number: episode.metadata.number,
            season: Cache::new(episode::season(&episode.metadata.title)),
            title: Cache::new(episode.metadata.title),
            published,
            length: Cache::empty(),
            thumbnail: Cache::empty(),
            note: Cache::empty(),
            panels: Cache::empty(),
            views: Some(episode.metadata.views),
            ad_status: Some(episode.dashboard_status.ad_status()),
            // TODO: look into the From impl for this as this might need to be try_from
            published_status: Some(episode.dashboard_status.into()),
        });
    }

    for page in 2..=pages {
        let dashboard_episodes = webtoon.client.get_episodes_dashboard(webtoon, page).await?;

        for episode in dashboard_episodes {
            let published = match episode
                .published
                .map(|timestamp| DateTime::from_timestamp_millis(timestamp))
            {
                Some(Some(published)) => Some(published),
                Some(None) => invariant!(
                    "`webtoons.com` should always return a valid unix millisecond timestamp, got: {:?}",
                    episode.published
                ),
                None => None,
            };

            episodes.insert(Episode {
                webtoon: webtoon.clone(),
                number: episode.metadata.number,
                season: Cache::new(episode::season(&episode.metadata.title)),
                title: Cache::new(episode.metadata.title),
                published,
                length: Cache::empty(),
                thumbnail: Cache::empty(),
                note: Cache::empty(),
                panels: Cache::empty(),
                views: Some(episode.metadata.views),
                ad_status: Some(episode.dashboard_status.ad_status()),
                published_status: Some(episode.dashboard_status.into()),
            });
        }

        // QUESTION: Maybe dont need this?
        // Sleep for one second to prevent getting a 429 response code for going between the pages too quickly.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // NOTE: we don't test on `invariant!(!episodes.is_empty())` as the dashboard could be of a brand new Webtoon.

    match u16::try_from(episodes.len()) {
        Ok(_) => {}
        Err(err) => {
            invariant!(
                "`webtoons.com` Webtoons should never have more than 65,535 episodes: {err}\n\ngot: {}",
                episodes.len()
            )
        }
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
