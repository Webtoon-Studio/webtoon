//! Represents an abstraction for `https://www.webtoons.com/*/originals`.

// mod genres;

use super::{Client, Webtoon, error::OriginalsError};
use crate::stdx::error::{Assume, assumption};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

pub(super) async fn scrape(client: &Client) -> Result<Vec<Webtoon>, OriginalsError> {
    // NOTE: Currently all languages follow this pattern.
    // TODO: Add tests for all languages.
    let selector = Selector::parse("ul.webtoon_list>li>a") //
        .assumption("`ul.webtoon_list>li>a` should be a valid selector")?;

    let days = [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "complete",
    ];

    // TODO: `Html` is not `Send`. Could add channels so that `scrape` becomes thread-safe.
    let documents: Vec<Html> = futures::future::try_join_all(
        days.iter()
            .map(|day| async { Ok::<Html, OriginalsError>(client.originals_page(day).await?) }),
    )
    .await?;

    let mut webtoons = Vec::with_capacity(2000);

    for html in documents {
        for card in html.select(&selector) {
            let href = card
                .attr("href")
                .assumption("html on `webtoons.com` Originals page should always have Webtoon card elements with `href` attributes in their `a` tag")?;

            let webtoon = match Webtoon::from_url_with_client(href, client) {
                Ok(webtoon) => webtoon,
                Err(err) => assumption!(
                    "urls gotten from `webtoons.com` Originals page should be valid urls for making a `Webtoon`: {err}"
                ),
            };

            webtoons.push(webtoon);
        }
    }

    assumption!(
        !webtoons.is_empty(),
        "after scraping `webtoons.com` Originals page, there should be at least some Webtoons that were found"
    );

    Ok(webtoons)
}

/// Represents a kind of release schedule for Originals.
///
/// For the days of the week, a Webtoon can have multiple.
///
/// If it's not a day of the week, it can only be either `Daily` or `Completed`, alone.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Schedule {
    /// Released on a single day of the week
    Weekday(Weekday),
    /// Released multiple days of the week
    Weekdays(Vec<Weekday>),
    /// Released daily
    Daily,
    /// Webtoon is completed
    Completed,
}

impl TryFrom<Vec<&str>> for Schedule {
    type Error = ParseScheduleError;

    fn try_from(releases: Vec<&str>) -> Result<Self, Self::Error> {
        match releases.as_slice() {
            [release] => [try_parse_weekday, try_parse_completed, try_parse_daily]
                .into_iter()
                .find_map(|parser| parser(release).ok())
                .ok_or_else(|| ParseScheduleError((*release).to_string())),
            // Multiple releases must be weekdays
            releases => releases
                .iter()
                .map(|release| Weekday::from_str(release))
                .collect::<Result<Vec<Weekday>, ParseScheduleError>>()
                .map(|weekdays| Self::Weekdays(weekdays)),
        }
    }
}

/// Represents a day of the week
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weekday {
    /// Released on Sunday
    Sunday,
    /// Released on Monday
    Monday,
    /// Released on Tuesday
    Tuesday,
    /// Released on Wednesday
    Wednesday,
    /// Released on Thursday
    Thursday,
    /// Released on Friday
    Friday,
    /// Released on Saturday
    Saturday,
}

/// An error which can happen when parsing a string to a schedule type.
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a `Schedule`")]
pub struct ParseScheduleError(String);

impl FromStr for Weekday {
    type Err = ParseScheduleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "MONDAY" | "MON" => Ok(Self::Monday),
            "TUESDAY" | "TUE" => Ok(Self::Tuesday),
            "WEDNESDAY" | "WED" => Ok(Self::Wednesday),
            "THURSDAY" | "THU" => Ok(Self::Thursday),
            "FRIDAY" | "FRI" => Ok(Self::Friday),
            "SATURDAY" | "SAT" => Ok(Self::Saturday),
            "SUNDAY" | "SUN" => Ok(Self::Sunday),
            _ => Err(ParseScheduleError(s.to_owned())),
        }
    }
}

impl FromStr for Schedule {
    type Err = ParseScheduleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "DAILY" => Ok(Self::Daily),
            "COMPLETED" => Ok(Self::Completed),
            _ => Err(ParseScheduleError(s.to_owned())),
        }
    }
}

// TODO: use `Schedule::from_str` for below parsing, as this is just duplicating logic.
fn try_parse_completed(release: &str) -> Result<Schedule, &str> {
    match release.trim() {
        "COMPLETED" => Ok(Schedule::Completed),
        release => Err(release),
    }
}

fn try_parse_daily(release: &str) -> Result<Schedule, &str> {
    match release.trim() {
        "DAILY" | "EVERYDAY" => Ok(Schedule::Daily),
        release => Err(release),
    }
}

fn try_parse_weekday(release: &str) -> Result<Schedule, &str> {
    let Ok(weekday) = Weekday::from_str(release.trim()) else {
        return Err(release.trim());
    };
    Ok(Schedule::Weekday(weekday))
}
