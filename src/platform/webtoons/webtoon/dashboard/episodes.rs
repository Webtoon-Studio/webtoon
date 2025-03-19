mod json;
use chrono::DateTime;
pub use json::*;
use tokio::sync::RwLock;

use crate::platform::webtoons::{Webtoon, errors::EpisodeError, webtoon::episode::Episode};
use std::{collections::HashSet, sync::Arc, time::Duration};

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

    let dashboard_episodes = DashboardEpisode::parse(&response)?;

    for episode in dashboard_episodes {
        episodes.insert(Episode {
            webtoon: webtoon.clone(),
            number: episode.metadata.number,
            season: Arc::new(RwLock::new(super::super::episode::season(
                &episode.metadata.title,
            ))),
            title: Arc::new(RwLock::new(Some(episode.metadata.title))),
            published: episode.published.map(|timestamp| {
                DateTime::from_timestamp_millis(timestamp)
                    .expect("webtoons should be using proper timestamps")
            }),
            page: Arc::new(RwLock::new(None)),
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
                season: Arc::new(RwLock::new(super::super::episode::season(
                    &episode.metadata.title,
                ))),
                title: Arc::new(RwLock::new(Some(episode.metadata.title))),
                published: episode.published.map(|timestamp| {
                    DateTime::from_timestamp_millis(timestamp)
                        .expect("webtoons should be using proper timestamps")
                }),
                page: Arc::new(RwLock::new(None)),
                views: Some(episode.metadata.views),
                ad_status: Some(episode.dashboard_status.ad_status()),
                published_status: Some(episode.dashboard_status.into()),
            });
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let mut episodes: Vec<Episode> = episodes.into_iter().collect();

    episodes.sort_unstable_by_key(Episode::number);

    Ok(episodes)
}

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
