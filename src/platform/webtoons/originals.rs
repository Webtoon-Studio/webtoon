//! Represents an abstraction for `https://www.webtoons.com/*/originals`.

use std::str::FromStr;

// mod genres;
use anyhow::{Context, anyhow};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Client, Language, Webtoon, errors::OriginalsError};

pub(super) async fn scrape(
    client: &Client,
    language: Language,
) -> Result<Vec<Webtoon>, OriginalsError> {
    // NOTE: Currently all languages follow this pattern
    let selector = Selector::parse("ul.webtoon_list>li>a") //
        .expect("`ul.webtoon_list>li>a` should be a valid selector");

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

    let documents: Vec<Html> = futures::future::try_join_all(days.iter().map(|day| async {
        Ok::<Html, OriginalsError>(client.get_originals_page(language, day).await?)
    }))
    .await?;

    let mut webtoons = Vec::with_capacity(2000);

    for html in documents {
        for card in html.select(&selector) {
            let href = card
                .attr("href")
                .context("`href` is missing, `a` tag should always have one")?;

            webtoons.push(Webtoon::from_url_with_client(href, client)?);
        }
    }

    if webtoons.is_empty() {
        return Err(OriginalsError::Unexpected(anyhow!(
            "Failed to scrape the originals page for webtoons"
        )));
    }

    Ok(webtoons)
}

/// Represents a kind of release schedule for Originals.
///
/// For the days of the week, a Webtoon can have multiple.
///
/// If its not a day of the week, it can only be either `Daily` or `Completed`, alone.
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
        if releases.len() == 1 {
            let release = releases
                .first()
                .expect("already checked that there is at least one element");

            if let Ok(weekday) = try_parse_weekday(release) {
                Ok(weekday)
            } else if let Ok(completed) = try_parse_completed(release) {
                Ok(completed)
            } else if let Ok(daily) = try_parse_daily(release) {
                Ok(daily)
            } else {
                Err(ParseScheduleError((*release).to_string()))
            }
        } else {
            // If there is more than one element it means that there are multiple days
            let mut weekdays = Vec::with_capacity(7);
            for release in releases {
                let weekday = Weekday::from_str(release)?;
                weekdays.push(weekday);
            }
            Ok(Self::Weekdays(weekdays))
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

/// An error which can happen when parsing a string to a release type.
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a `Release`")]
pub struct ParseScheduleError(String);

impl FromStr for Weekday {
    type Err = ParseScheduleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "MONDAY"
            | "MON"
            | "MONTAG"
            | "MO"
            | "LUNDIS"
            | "LUN"
            | "LUNS"
            | "LUNES"
            | "SENIN"
            | "SEN"
            | "วันจันทร์"
            | "จันทร์"
            | "週一" => Ok(Self::Monday),
            "TUESDAY"
            | "TUE"
            | "DIENSTAG"
            | "DI"
            | "MARDIS"
            | "MAR"
            | "MARS"
            | "MARTES"
            | "SELASA"
            | "SEL"
            | "วันอังคาร"
            | "อังคาร"
            | "週二" => Ok(Self::Tuesday),
            "WEDNESDAY" | "WED" | "MITTWOCH" | "MI" | "MERCREDIS" | "MER" | "MERS"
            | "MIÉRCOLES" | "MIÉ" | "RABU" | "RAB" | "วันพุธ" | "พุธ" | "週三" => {
                Ok(Self::Wednesday)
            }
            "THURSDAY"
            | "THU"
            | "DONNERSTAG"
            | "DO"
            | "JEUDIS"
            | "JEU"
            | "JEUS"
            | "JUEVES"
            | "JUE"
            | "KAMIS"
            | "KAM"
            | "วันพฤหัสบดี"
            | "พฤหัสบดี"
            | "週四" => Ok(Self::Thursday),
            "FRIDAY"
            | "FRI"
            | "FREITAG"
            | "FR"
            | "VENDREDIS"
            | "VEN"
            | "VENS"
            | "VIERNES"
            | "VIE"
            | "JUMAT"
            | "JUM"
            | "วันศุกร์"
            | "ศุกร์"
            | "週五" => Ok(Self::Friday),
            "SATURDAY"
            | "SAT"
            | "SAMSTAG"
            | "SA"
            | "SAMEDIS"
            | "SAM"
            | "SAMS"
            | "SÁBADOS"
            | "SÁB"
            | "SABTU"
            | "SAB"
            | "วันเสาร์"
            | "เสาร์"
            | "週六" => Ok(Self::Saturday),
            "SUNDAY"
            | "SUN"
            | "SONNTAG"
            | "SO"
            | "DIMANCHES"
            | "DIM"
            | "DIMS"
            | "DOMINGOS"
            | "DOM"
            | "MINGGU"
            | "MIN"
            | "วันอาทิตย์"
            | "อาทิตย์"
            | "週日" => Ok(Self::Sunday),
            _ => Err(ParseScheduleError(s.to_owned())),
        }
    }
}

impl FromStr for Schedule {
    type Err = ParseScheduleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "DAILY" | "TÄGLICH" | "JOURS" | "ทุกวัน" | "每日" => Ok(Self::Daily),
            "COMPLETED"
            | "COM"
            | "ABGESCHLOSSEN"
            | "TERMINÉ"
            | "FINALIZADAS"
            | "ฟรีทุกวัน"
            | "完結" => Ok(Self::Completed),
            _ => Err(ParseScheduleError(s.to_owned())),
        }
    }
}

fn try_parse_completed(release: &str) -> Result<Schedule, &str> {
    match release.trim() {
        "COMPLETED" | "COM" | "ABGESCHLOSSEN" | "TERMINÉ" | "FINALIZADAS" | "ฟรีทุกวัน" | "完結" => {
            Ok(Schedule::Completed)
        }
        release => Err(release),
    }
}

fn try_parse_daily(release: &str) -> Result<Schedule, &str> {
    match release.trim() {
        "DAILY" | "TÄGLICH" | "JOURS" | "ทุกวัน" | "每日" => Ok(Schedule::Daily),
        release => Err(release),
    }
}

fn try_parse_weekday(release: &str) -> Result<Schedule, &str> {
    let Ok(weekday) = Weekday::from_str(release.trim()) else {
        return Err(release.trim());
    };
    Ok(Schedule::Weekday(weekday))
}
