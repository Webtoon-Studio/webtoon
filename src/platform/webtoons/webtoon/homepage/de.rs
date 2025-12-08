use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use parking_lot::RwLock;
use scraper::{ElementRef, Html, Selector};
use std::sync::Arc;

use super::{
    super::episode::{self, PublishedStatus},
    Page,
};
use crate::platform::webtoons::{
    Webtoon,
    errors::{Invariant, invariant},
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
            schedule: Some(schedule(html)?),
            thumbnail: None,
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
            schedule: None,
            thumbnail: Some(super::en::thumbnail(html)?),
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
        // First occurrence of `em.cnt` is for the views.
        .next()
        .invariant("`em.cnt`(views) on `webtoons.com` Webtoon homepage is missing: webtoons page displays total views")?
        .inner_html();

    // TODO: There is no German Webtoon with a billion views, so unknown how it would be represented.
    match views.as_str() {
        millions if millions.ends_with('M') => {
            let (millionth, hundred_thousandth) = millions
                .trim_end_matches('M')
                .split_once(',')
                .invariant("on german `webtoons.com` Webtoon homepage, a million views is always represented as a decimal value, with an `M` suffix and a comma separator, eg. `1,3M`, and so should always split on `,`")?;

            let millions = millionth.parse::<u64>()
                .invariant(format!("`on the german `webtoons.com` Webtoon homepage, the millions part of the views count should always fit in a `u64`, got: {millionth}"))?;

            let hundred_thousands = hundred_thousandth.parse::<u64>()
                .invariant(format!("`on the german `webtoons.com` Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u64`, got: {hundred_thousandth}"))?;

            Ok((millions * 1_000_000) + (hundred_thousands * 100_000))
        }
        thousands_or_less => Ok(thousands_or_less
            .replace('.', "")
            .parse::<u64>()
            .invariant(format!("hundreds to hundreds of thousands of subscribers should fit in a `u64`, got: {thousands_or_less}"))?),
    }
}

fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let subscribers = html
        .select(&selector)
        // First instance of `em.cnt` is for views.
        .nth(1)
        .invariant("second instance of `em.cnt`(subscribers) on german `webtoons.com` Webtoon homepage is missing")?
        .inner_html();

    invariant!(
        !subscribers.is_empty(),
        "subscriber element(`em.cnt`) on german `webtoons.com` Webtoon homepage should never be empty"
    );

    match subscribers.as_str() {
        millions if millions.ends_with('M') => {
            let (millionth, hundred_thousandth) = millions
                .trim_end_matches('M')
                // 4,2M
                .split_once(',')
                .invariant("on `webtoons.com` german Webtoon homepage, a million subscribers is always represented as a decimal value, with an `M` suffix, eg. `1,3M`, and so should always split on `.`")?;

            let millions = millionth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` german Webtoon homepage, the millions part of the subscribers count should always fit in a `u32`, got: {millionth}"))?;

            let hundred_thousands = hundred_thousandth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` german Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundred_thousandth}"))?;

            Ok((millions * 1_000_000) + (hundred_thousands * 100_000))
        }
        thousands_or_less => {
            let (thousandth, hundreth) = thousands_or_less
                .split_once(',')
                .invariant("on `webtoons.com` german Webtoon homepage, a <1,000,000 subscribers is always represented as a decimal value, eg. `469.035`, and so should always split on `.`")?;

            let thousands = thousandth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` german Webtoon homepage, the thousands part of the subscribers count should always fit in a `u32`, got: {thousandth}"))?;

            let hundreds = hundreth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` german Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundreth}"))?;

            Ok((thousands * 1_000) + hundreds)
        }
    }
}

fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info") //
        .expect("`p.day_info` should be a valid selector");

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .invariant("`p.day_info`(schedule) on german `webtoons.com` originals Webtoons is missing")?
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

    invariant!(
        !releases.is_empty(),
        "original Webtoon homepage on german `webtoons.com` should always have a release schedule, even if completed"
    );

    let schedule = match Schedule::try_from(releases) {
        Ok(schedule) => schedule,
        Err(err) => invariant!(
            "german originals on `webtoons.com` should only have a few known release days/types: {err}"
        ),
    };

    Ok(schedule)
}

pub(super) fn episode(
    element: &ElementRef<'_>,
    webtoon: &Webtoon,
) -> Result<Episode, WebtoonError> {
    let title = super::en::episode_title(element)?;

    let data_episode_no = element
        .value()
        .attr("data-episode-no")
        .invariant(
            "`data-episode-no` attribute should be found on german `webtoons.com` Webtoon homepage, representing the episodes number",
        )?;

    let number = data_episode_no
        .parse::<u16>()
        .invariant(format!("`data-episode-no` on german `webtoons.com` should be parse into a `u16`, but got: {data_episode_no}"))?;

    let published = episode_published_date(element)?;

    Ok(Episode {
        webtoon: webtoon.clone(),
        season: Arc::new(RwLock::new(episode::season(&title))),
        title: Arc::new(RwLock::new(Some(title))),
        number,
        published: Some(published),
        published_status: Some(PublishedStatus::Published),

        length: Arc::new(RwLock::new(None)),
        thumbnail: Arc::new(RwLock::new(None)),
        note: Arc::new(RwLock::new(None)),
        panels: Arc::new(RwLock::new(None)),
        views: None,
        ad_status: None,
    })
}

fn episode_published_date(episode: &ElementRef<'_>) -> Result<DateTime<Utc>, WebtoonError> {
    let selector = Selector::parse("span.date") //
        .expect("`span.date` should be a valid selector");

    let text = episode
        .select(&selector)
        .next()
        .invariant("`span.date` should be found on german `webtoons.com` Webtoon homepage with episodes listed on it")?
        .text()
        .next()
        .invariant("`span.date` on german `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    let date = NaiveDate::parse_from_str(text, "%d.%m.%Y")
        .invariant(format!("the german `webtoons.com` Webtoon homepage episode date should follow the `24.12.2021` format, got: {text}"))?;

    let time = NaiveTime::from_hms_opt(2, 0, 0).expect("2:00:00 should be a valid `NaiveTime`");

    Ok(DateTime::from_naive_utc_and_offset(
        date.and_time(time),
        Utc,
    ))
}
