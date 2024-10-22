mod de;
mod en;
mod es;
mod fr;
mod id;
mod th;
mod zh;

use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

use crate::platform::webtoons::{
    creator::Creator,
    meta::{Genre, Language},
    originals::Release,
    Webtoon,
};

use super::{episode::Episode, WebtoonError};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Page {
    title: String,
    creators: Vec<Creator>,
    genres: Vec<Genre>,
    summary: String,
    views: u64,
    subscribers: u32,
    rating: f64,
    release: Option<Vec<Release>>,
    thumbnail: Url,
    banner: Option<Url>,
    pages: u8,
}

#[inline]
pub async fn scrape<'a>(webtoon: &Webtoon) -> Result<Page, WebtoonError> {
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
    pub(crate) fn rating(&self) -> f64 {
        self.rating
    }

    #[inline]
    pub(crate) fn release(&self) -> Option<&[Release]> {
        self.release.as_deref()
    }

    #[inline]
    pub(crate) fn thumbnail(&self) -> &str {
        self.thumbnail.as_str()
    }

    #[inline]
    pub(crate) fn banner(&self) -> Option<&str> {
        self.banner.as_ref().map(Url::as_str)
    }
}

pub(super) async fn episodes(webtoon: &Webtoon) -> Result<Vec<Episode>, WebtoonError> {
    // TODO: If it ever becomes possible to detect the last page via a redirect or some other mechanism, the initial
    // scrape shouldn't be needed anymore, and can just be iterated over with `1..` until the last page

    // IDEA: Might be able to make in iterator instead that would yield an `Episode` using the `Episode::exists` function.
    // This would better support hidden episodes, like those behind ads, as well as for completed webtoons, who's episodes
    // list can be truncated.

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

        // This page never returns a rate limt response, it just silently fails, leading to missed pages.
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    // NOTE: Episodes are scraped from newest to oldest, this will make the returned Vec oldest first.
    episodes.reverse();

    Ok(episodes)
}
