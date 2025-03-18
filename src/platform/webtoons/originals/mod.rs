//! Represents an abstraction for the `https://www.webtoons.com/*/originals` endpoint.

use std::str::FromStr;

// mod genres;
use anyhow::Context;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Client, Language, Webtoon, errors::OriginalsError};

pub(super) async fn scrape(
    client: &Client,
    language: Language,
) -> Result<Vec<Webtoon>, OriginalsError> {
    // NOTE: Currently all languages follow this pattern
    let selector = Selector::parse("ul.daily_card>li>a") //
        .expect("`ul.daily_card>li>a` should be a valid selector");

    let mut webtoons = Vec::with_capacity(1000);

    let document = client.get_originals_page(language).await?.text().await?;

    let html = Html::parse_document(&document);

    // TODO: Need to access the `genres` page as the home page of the webtoons only contains one genre when there could
    // be many more.
    // let genres = genres::scrape(client).await?;

    for card in html.select(&selector) {
        let href = card
            .attr("href")
            .context("`href` is missing, `a` tag should always have one")?;

        webtoons.push(Webtoon::from_url_with_client(href, client)?);
    }

    Ok(webtoons)
}

/// Represents a kind of release schedule for Originals.  
///
/// For the days of the week, a webtoon can have multiple.
///
/// If its not a day of the week, it can only be either `Daily` or `Completed`, alone.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Release {
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
    /// Released daily
    Daily,
    /// Webtoon is completed
    Completed,
}

/// An error which can happen when parsing a string to a [`Release`].
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a `Release`")]
pub struct ParseReleaseError(String);

impl FromStr for Release {
    type Err = ParseReleaseError;
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
            "DAILY" | "TÄGLICH" | "JOURS" | "ทุกวัน" | "每日" => Ok(Self::Daily),
            "COMPLETED"
            | "COM"
            | "ABGESCHLOSSEN"
            | "TERMINÉ"
            | "FINALIZADAS"
            | "ฟรีทุกวัน"
            | "完結" => Ok(Self::Completed),
            _ => Err(ParseReleaseError(s.to_owned())),
        }
    }
}
