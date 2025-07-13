mod de;
mod en;
mod es;
mod fr;
mod id;
mod th;
mod zh;

use anyhow::anyhow;
use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

use crate::platform::webtoons::{
    Webtoon,
    creator::Creator,
    meta::{Genre, Language},
    originals::Schedule,
};

use super::{WebtoonError, episode::Episode};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Page {
    title: String,
    creators: Vec<Creator>,
    genres: Vec<Genre>,
    summary: String,
    views: u64,
    subscribers: u32,
    schedule: Option<Schedule>,
    thumbnail: Option<Url>,
    banner: Option<Url>,
    pages: u16,
}

#[inline]
pub async fn scrape(webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let response = webtoon.client.get_webtoon_page(webtoon, None).await?;

    let document = response.text().await?;

    let html = Html::parse_document(&document);

    let page = match webtoon.language {
        Language::En => en::page(&html, webtoon)?,
        Language::Zh => zh::page(&html, webtoon)?,
        Language::Th => th::page(&html, webtoon)?,
        Language::Id => id::page(&html, webtoon)?,
        Language::Es => es::page(&html, webtoon)?,
        Language::Fr => fr::page(&html, webtoon)?,
        Language::De => de::page(&html, webtoon)?,
    };

    Ok(page)
}

impl Page {
    #[inline]
    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub(crate) fn creators(&self) -> &[Creator] {
        &self.creators
    }

    #[inline]
    pub(crate) fn genres(&self) -> &[Genre] {
        &self.genres
    }

    #[inline]
    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    #[inline]
    pub(crate) fn views(&self) -> u64 {
        self.views
    }

    #[inline]
    pub(crate) fn subscribers(&self) -> u32 {
        self.subscribers
    }

    #[inline]
    pub(crate) fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    #[inline]
    pub(crate) fn thumbnail(&self) -> Option<&str> {
        self.thumbnail.as_ref().map(|url| url.as_str())
    }

    #[inline]
    pub(crate) fn banner(&self) -> Option<&str> {
        self.banner.as_ref().map(Url::as_str)
    }
}

pub(super) async fn episodes(webtoon: &Webtoon) -> Result<Vec<Episode>, WebtoonError> {
    let page = scrape(webtoon).await?;

    let pages = page.pages;

    // NOTE: currently all languages use this for the list element; this could change.
    let selector = Selector::parse("li._episodeItem") //
        .expect("`li._episodeItem` should be a valid selector");

    let mut episodes = Vec::with_capacity(pages as usize * 10);

    for page in 1..=pages {
        let response = webtoon.client.get_webtoon_page(webtoon, Some(page)).await?;

        let html = Html::parse_document(&response.text().await?);

        for element in html.select(&selector) {
            let episode = match webtoon.language {
                Language::En => en::episode(&element, webtoon)?,
                Language::Zh => zh::episode(&element, webtoon)?,
                Language::Th => th::episode(&element, webtoon)?,
                Language::Id => id::episode(&element, webtoon)?,
                Language::Es => es::episode(&element, webtoon)?,
                Language::Fr => fr::episode(&element, webtoon)?,
                Language::De => de::episode(&element, webtoon)?,
            };

            episodes.push(episode);
        }

        // Sleep for one second to prevent getting a 429 response code for going between the pages to quickly.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // NOTE: Consistently return by episode order
    episodes.sort_by(|a, b| a.number.cmp(&b.number));

    Ok(episodes)
}

pub(super) async fn first_episode(webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
    let page = scrape(webtoon).await?.pages;

    // NOTE: currently all languages use this for the list element; this could change.
    let selector = Selector::parse("li._episodeItem") //
        .expect("`li._episodeItem` should be a valid selector");

    let response = webtoon.client.get_webtoon_page(webtoon, Some(page)).await?;

    let html = Html::parse_document(&response.text().await?);

    let mut first: Option<Episode> = None;

    for element in html.select(&selector) {
        let episode = match webtoon.language {
            Language::En => en::episode(&element, webtoon)?,
            Language::Zh => zh::episode(&element, webtoon)?,
            Language::Th => th::episode(&element, webtoon)?,
            Language::Id => id::episode(&element, webtoon)?,
            Language::Es => es::episode(&element, webtoon)?,
            Language::Fr => fr::episode(&element, webtoon)?,
            Language::De => de::episode(&element, webtoon)?,
        };

        first = Some(episode);
    }

    first.ok_or_else(|| {
        anyhow!("no episode was found on public webtoon, which shouldn't be possible").into()
    })
}
