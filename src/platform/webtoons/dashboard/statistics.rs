use anyhow::{Context, anyhow};
use scraper::{Html, Selector};

use crate::platform::webtoons::{Webtoon, errors::WebtoonError};

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Stats {
    pub updates: u16,
    pub subscribers: u32,
    pub this_month: Current,
    pub last_month: Previous,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Current {
    pub updates: u8,
    pub monthly_views: u32,
    pub daily_views: u32,
    pub average_views_per_update: u32,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Previous {
    pub updates: Option<u8>,
    pub monthly_views: Option<u32>,
    pub average_views_per_update: Option<u32>,
}

pub async fn scrape(webtoon: &Webtoon) -> Result<Stats, WebtoonError> {
    let html = webtoon.client.get_stats_dashboard(webtoon).await?;

    Ok(Stats {
        subscribers: subscribers(&html)?,
        ..Default::default()
    })
}

fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let subscribers_text_selector =
        Selector::parse(r".col3>p").expect("failed to parse subscriber descriptor selector");

    let text = html
        .select(&subscribers_text_selector)
        .next()
        .context("`.col3>p` is missing: stats dashboard should have a subscribers element")?
        .text()
        .next()
        .context("`.col3>p` was found but no text was present")?;

    if text != "Subscribers" {
        return Err(WebtoonError::Unexpected(anyhow!(
            "column was not a subscribers column but instead: `{text}`"
        )));
    }

    let subscribers_selector =
        Selector::parse(r".col3>.num").expect("failed to parse subscriber selector");

    let count = html
        .select(&subscribers_selector)
        .next()
        .context("`.col3>.num` is missing: subscriber column should have a number count")?
        .text()
        .next()
        .context("`.col3>.num` was found but no text was present")?;

    let subscribers = match count {
        million if million.ends_with('M') => {
            let millions = &million[..million.len() - 1]
                .parse::<f64>()
                .map_err(|err| WebtoonError::Unexpected(err.into()))?;

            (millions * 1_000_000.0) as u32
        }
        thousand => thousand
            .replace(',', "")
            .parse::<u32>()
            .map_err(|err| WebtoonError::Unexpected(err.into()))?,
    };

    Ok(subscribers)
}
