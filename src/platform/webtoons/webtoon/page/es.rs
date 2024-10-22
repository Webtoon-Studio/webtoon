use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use tokio::sync::Mutex;

use super::Page;
use crate::platform::webtoons::{
    meta::Scope,
    originals::Release,
    webtoon::{episode::Episode, WebtoonError},
    Webtoon,
};

pub(super) fn page(html: &Html, webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let page = match webtoon.scope {
        Scope::Original(_) => Page {
            title: super::en::title(html)?,
            creators: super::en::creators(html, &webtoon.client)?,
            genres: super::en::genres(html)?,
            summary: super::en::summary(html)?,
            views: super::en::views(html)?,
            subscribers: super::en::subscribers(html)?,
            rating: super::en::rating(html)?,
            release: Some(release(html)?),
            thumbnail: super::en::original_thumbnail(html)?,
            banner: Some(super::en::banner(html)?),
            pages: super::en::calculate_total_pages(html)?,
        },
        Scope::Canvas => Page {
            title: super::en::title(html)?,
            creators: super::en::creators(html, &webtoon.client)?,
            genres: super::en::genres(html)?,
            summary: super::en::summary(html)?,
            views: super::en::views(html)?,
            subscribers: super::en::subscribers(html)?,
            rating: super::en::rating(html)?,
            release: None,
            thumbnail: super::en::canvas_thumbnail(html)?,
            banner: Some(super::en::banner(html)?),
            pages: super::en::calculate_total_pages(html)?,
        },
    };

    Ok(page)
}

fn release(html: &Html) -> Result<Vec<Release>, WebtoonError> {
    let selector = Selector::parse(r"p.day_info").expect("`p.day_info` should be a valid selector");

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .context("`p.day_info` is missing: webtoons displays a release schedule")?
        .text()
    {
        if text == "NUEVO" {
            continue;
        }

        for release in text.split_whitespace() {
            // `LUN,` -> `LUN`
            let release = release.trim_end_matches(',');

            if matches!(release, "TODOS" | "LOS") {
                continue;
            }

            releases.push(
                Release::from_str(release).map_err(|err| WebtoonError::Unexpected(err.into()))?,
            );
        }
    }

    Ok(releases)
}

pub(super) fn episode(element: &ElementRef, webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
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
        season: Arc::new(Mutex::new(super::super::episode::season(&title))),
        title: Arc::new(Mutex::new(Some(title))),
        number,
        published: Some(published),
        page: Arc::new(Mutex::new(None)),
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

    let mut parts = episode
        .select(&selector)
        .next()
        .context("`span.date` should be found on a webtoon page with episodes on it")?
        .text()
        .next()
        .context("`span.date` should have text inside it")?
        .split_whitespace();

    let day = parts
        .next()
        .context("spanish date did not have a day number")?;

    let month = parts
        .next()
        .map(|part| match part {
            // Jan
            "ene." => Ok("1"),
            // Feb
            "feb." => Ok("2"),
            // Mar
            "mar." => Ok("3"),
            // Apr
            "abr." => Ok("4"),
            // May
            "may." => Ok("5"),
            // Jun
            "jun." => Ok("6"),
            // Jul
            "jul." => Ok("7"),
            // Aug
            "ago." => Ok("8"),
            // Sep
            "sept." => Ok("9"),
            // Oct
            "oct." => Ok("10"),
            // Nov
            "nov." => Ok("11"),
            // Dec
            "dic." => Ok("12"),
            _ => anyhow::bail!("unknown spanish month abriviation: {part}"),
        })
        .transpose()?
        .context("spanish date did not have a month abbreviation")?;

    let year = parts
        .next()
        .context("spanish date did not have a year number")?;

    let mut date = String::with_capacity(32);

    date.push_str(day);
    date.push(' ');
    date.push_str(month);
    date.push(' ');
    date.push_str(year);
    date.push(' ');
    date.push_str("02:00:00 +0000");

    let date = DateTime::parse_from_str(&date, "%d %m %Y %T %z")
        .context("spanish webtoon page should have dates of pattern `14 ก.ค. 2024`")?;

    Ok(date.into())
}
