use anyhow::Context;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use scraper::{ElementRef, Html, Selector};
use std::sync::Arc;

use super::Page;
use crate::platform::webtoons::{
    Webtoon,
    meta::Scope,
    originals::Schedule,
    webtoon::{WebtoonError, episode::Episode},
};

pub(super) fn page(html: &Html, webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let page = match webtoon.scope {
        Scope::Original(_) => Page {
            title: super::en::title(html)?,
            creators: super::en::creators(html, &webtoon.client)?,
            genres: super::en::genres(html)?,
            summary: super::en::summary(html)?,
            views: views(html)?,
            subscribers: subscribers(html)?,
            rating: rating(html)?,
            schedule: Some(schedule(html)?),
            thumbnail: super::en::original_thumbnail(html)?,
            banner: Some(super::en::banner(html)?),
            pages: super::en::calculate_total_pages(html)?,
        },
        Scope::Canvas => Page {
            title: super::en::title(html)?,
            creators: super::en::creators(html, &webtoon.client)?,
            genres: super::en::genres(html)?,
            summary: super::en::summary(html)?,
            views: views(html)?,
            subscribers: subscribers(html)?,
            rating: rating(html)?,
            schedule: None,
            thumbnail: super::en::canvas_thumbnail(html)?,
            banner: Some(super::en::banner(html)?),
            pages: super::en::calculate_total_pages(html)?,
        },
    };

    Ok(page)
}

fn views(html: &Html) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let views = html
        .select(&selector)
        .next()
        .context("`em.cnt` is missing: webtoons page displays total views")?
        .inner_html();

    match views.as_str() {
        million if million.ends_with('M') => {
            let value = million
                .replace(',', ".")
                .trim_end_matches('M')
                .parse::<f64>()
                .map_err(|err| WebtoonError::Unexpected(err.into()))
                .context(views)?;

            Ok((value * 1_000_000.0) as u64)
        }
        thousand => Ok(thousand.replace('.', "").parse::<u64>().context(views)?),
    }
}

fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let subscribers = html
        .select(&selector)
        .nth(1)
        .context("`em.cnt` is missing: webtoons page displays subscribers")?
        .inner_html();

    match subscribers.as_str() {
        million if million.ends_with('M') => {
            let value = million
                .replace(',', ".")
                .trim_end_matches('M')
                .parse::<f64>()
                .context(subscribers)?;

            Ok((value * 1_000_000.0) as u32)
        }
        thousand => Ok(thousand
            .replace('.', "")
            .parse::<u32>()
            .context(subscribers)?),
    }
}

fn rating(html: &Html) -> Result<f64, WebtoonError> {
    let selector = Selector::parse(r"em#_starScoreAverage") //
        .expect("`em#_starScoreAverage` should be a valid selector");

    let rating = html
        .select(&selector)
        .next()
        .context("`em#_starScoreAverage` is missing: webtoons displays the rating")?
        .text()
        .next()
        .context("`em#_starScoreAverage` was found but no text was present")?
        .replace(',', ".");

    Ok(rating.parse::<f64>().context(rating)?)
}

fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info").expect("`p.day_info` should be a valid selector");

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .context("`p.day_info` is missing: webtoons displays a release schedule")?
        .text()
    {
        if text == "UP" {
            continue;
        }

        for release in text.split_whitespace() {
            // `MO,` -> `MO`
            let release = release.trim_end_matches(',');

            if release == "IMMER" {
                continue;
            }

            releases.push(release);
        }
    }

    let schedule = Schedule::try_from(releases) //
        .map_err(|err| WebtoonError::Unexpected(err.into()))?;

    Ok(schedule)
}

pub(super) fn episode(
    element: &ElementRef<'_>,
    webtoon: &Webtoon,
) -> Result<Episode, WebtoonError> {
    let title = super::en::episode_title(element)?;

    let number = element
        .value()
        .attr("data-episode-no")
        .context("attribute `data-episode-no` should be found on webtoon page with episodes on it")?
        .parse::<u16>()
        .context("`data-episode-no` was not an int")?;

    let published = episode_published_date(element)?;

    Ok(Episode {
        webtoon: webtoon.clone(),
        season: Arc::new(RwLock::new(super::super::episode::season(&title))),
        title: Arc::new(RwLock::new(Some(title))),
        number,
        published: Some(published),
        page: Arc::new(RwLock::new(None)),
        views: None,
        // NOTE: Impossible to say from this page. In general any random Original episode would have been
        // behind an ad, but the initial release episodes which never were would be impossible to tell.
        ad_status: None,
        published_status: Some(super::super::episode::PublishedStatus::Published),
    })
}

// NOTE: Currently forces all dates to be at 02:00 UTC as thats when the originals get released.
// For more accurate times, must have a session.
fn episode_published_date(episode: &ElementRef<'_>) -> Result<DateTime<Utc>, WebtoonError> {
    let selector = Selector::parse("span.date") //
        .expect("`span.date` should be a valid selector");

    let mut date = episode
        .select(&selector)
        .next()
        .context("`span.date` should be found on a webtoon page with episodes on it")?
        .text()
        .next()
        .context("`span.date` should have text inside it")?
        .trim()
        .to_string();

    date.push_str(" 02:00:00 +0000");

    let date = DateTime::parse_from_str(&date, "%d.%m.%Y %T %z")
        .context("german webtoon page should have dates of pattern `2024年9月15日`")?;

    Ok(date.into())
}
