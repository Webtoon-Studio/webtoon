use crate::platform::webtoons::{Webtoon, error::SessionError, webtoon::episode::Episode};
use assumptions::Assumption;

use std::debug_assert as ensure;

pub async fn episodes(webtoon: &Webtoon) -> Result<Vec<Episode>, SessionError> {
    let series_analytics = webtoon.client.fetch_series_analytics(webtoon, 1).await?;

    let pages = series_analytics.total_pages;
    let count = series_analytics.total_count as usize;

    let mut episodes = Vec::with_capacity(count);

    for episode in series_analytics.episodes {
        episodes.push(Episode::try_from((webtoon, episode))?);
    }

    for page in 2..=pages {
        let series_analytics = webtoon.client.fetch_series_analytics(webtoon, page).await?;

        for episode in series_analytics.episodes {
            episodes.push(Episode::try_from((webtoon, episode))?);
        }
    }

    ensure!(
        episodes.len() == count,
        "total episodes should match the episode count from the dashboard"
    );

    Ok(episodes)
}

#[expect(unused, clippy::todo)]
pub fn subscribers(webtoon: &Webtoon) -> Result<u32, Assumption> {
    todo!()
}

#[expect(unused, clippy::todo)]
pub fn views(webtoon: &Webtoon) -> Result<u32, Assumption> {
    todo!()
}
