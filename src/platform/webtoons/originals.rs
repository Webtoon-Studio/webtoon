//! Originals page at `https://www.webtoons.com/en/originals`.

use super::{Client, Webtoon, error::OriginalsError};
use crate::stdx::error::{Assume, assume};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

pub(super) async fn scrape(client: &Client) -> Result<Vec<Webtoon>, OriginalsError> {
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

    let documents: Vec<Html> = futures::future::try_join_all(days.iter().map(|&day| async {
        let page = client.fetch_originals_page(day).await?;
        Ok::<Html, OriginalsError>(page)
    }))
    .await?;

    let mut webtoons = Vec::with_capacity(2000);

    for html in documents {
        for card in html.select(&selector) {
            let href = card
                .attr("href")
                .assumption("html on `webtoons.com` Originals page should always have Webtoon card elements with `href` attributes in their `a` tag")?;

            let webtoon =  Webtoon::from_url_with_client(href, client)
                .assumption("urls gotten from `webtoons.com` Originals page should be valid urls for making a `Webtoon`")?;

            webtoons.push(webtoon);
        }
    }

    assume!(
        !webtoons.is_empty(),
        "after scraping `webtoons.com` Originals page, there should be at least some Webtoons that were found"
    );

    Ok(webtoons)
}

/// The release schedule for an `Original` webtoon.
///
/// A webtoon can release on one or more days of the week, daily, or be completed.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Schedule {
    /// Released on a single day of the week.
    Weekday(Weekday),
    /// Released on multiple days of the week.
    Weekdays(Vec<Weekday>),
    /// Released daily.
    Daily,
    /// The webtoon has completed its run.
    Completed,
}

impl TryFrom<Vec<&str>> for Schedule {
    type Error = ParseScheduleError;

    /// # Invariant
    ///
    /// Callers must ensure that `releases` is not empty.
    #[inline]
    fn try_from(releases: Vec<&str>) -> Result<Self, Self::Error> {
        debug_assert!(!releases.is_empty());

        match releases.as_slice() {
            [release] => Self::from_str(release),
            releases => releases
                .iter()
                .map(|release| Weekday::from_str(release))
                .collect::<Result<Vec<Weekday>, _>>()
                .map(Schedule::Weekdays),
        }
    }
}

/// A day of the week on which a webtoon releases.
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

    #[inline]
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

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "DAILY" | "EVERYDAY" => Ok(Self::Daily),
            "COMPLETED" => Ok(Self::Completed),
            _ => Weekday::from_str(s).map(Self::Weekday),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn weekday_parses_long_form() {
        assert_eq!(Weekday::from_str("MONDAY").unwrap(), Weekday::Monday);
        assert_eq!(Weekday::from_str("TUESDAY").unwrap(), Weekday::Tuesday);
        assert_eq!(Weekday::from_str("WEDNESDAY").unwrap(), Weekday::Wednesday);
        assert_eq!(Weekday::from_str("THURSDAY").unwrap(), Weekday::Thursday);
        assert_eq!(Weekday::from_str("FRIDAY").unwrap(), Weekday::Friday);
        assert_eq!(Weekday::from_str("SATURDAY").unwrap(), Weekday::Saturday);
        assert_eq!(Weekday::from_str("SUNDAY").unwrap(), Weekday::Sunday);
    }

    #[test]
    fn weekday_parses_short_form() {
        assert_eq!(Weekday::from_str("MON").unwrap(), Weekday::Monday);
        assert_eq!(Weekday::from_str("TUE").unwrap(), Weekday::Tuesday);
        assert_eq!(Weekday::from_str("WED").unwrap(), Weekday::Wednesday);
        assert_eq!(Weekday::from_str("THU").unwrap(), Weekday::Thursday);
        assert_eq!(Weekday::from_str("FRI").unwrap(), Weekday::Friday);
        assert_eq!(Weekday::from_str("SAT").unwrap(), Weekday::Saturday);
        assert_eq!(Weekday::from_str("SUN").unwrap(), Weekday::Sunday);
    }

    #[test]
    fn weekday_trims_whitespace() {
        assert_eq!(Weekday::from_str("  MONDAY  ").unwrap(), Weekday::Monday);
    }

    #[test]
    fn weekday_errors_on_unknown() {
        assert!(Weekday::from_str("FUNDAY").is_err());
        assert!(Weekday::from_str("").is_err());
    }

    #[test]
    fn schedule_parses_daily() {
        assert_eq!(Schedule::from_str("DAILY").unwrap(), Schedule::Daily);
        assert_eq!(Schedule::from_str("EVERYDAY").unwrap(), Schedule::Daily);
    }

    #[test]
    fn schedule_parses_completed() {
        assert_eq!(
            Schedule::from_str("COMPLETED").unwrap(),
            Schedule::Completed
        );
    }

    #[test]
    fn schedule_parses_weekday() {
        assert_eq!(
            Schedule::from_str("MONDAY").unwrap(),
            Schedule::Weekday(Weekday::Monday)
        );
    }

    #[test]
    fn schedule_trims_whitespace() {
        assert_eq!(Schedule::from_str("  DAILY  ").unwrap(), Schedule::Daily);
    }

    #[test]
    fn schedule_errors_on_unknown() {
        assert!(Schedule::from_str("FUNDAY").is_err());
        assert!(Schedule::from_str("").is_err());
    }

    #[test]
    fn schedule_try_from_single_weekday() {
        assert_eq!(
            Schedule::try_from(vec!["MONDAY"]).unwrap(),
            Schedule::Weekday(Weekday::Monday)
        );
    }

    #[test]
    fn schedule_try_from_daily() {
        assert_eq!(Schedule::try_from(vec!["DAILY"]).unwrap(), Schedule::Daily);
    }

    #[test]
    fn schedule_try_from_completed() {
        assert_eq!(
            Schedule::try_from(vec!["COMPLETED"]).unwrap(),
            Schedule::Completed
        );
    }

    #[test]
    fn schedule_try_from_multiple_weekdays() {
        assert_eq!(
            Schedule::try_from(vec!["MONDAY", "WEDNESDAY", "FRIDAY"]).unwrap(),
            Schedule::Weekdays(vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday])
        );
    }

    #[test]
    fn schedule_try_from_errors_on_unknown_single() {
        assert!(Schedule::try_from(vec!["FUNDAY"]).is_err());
    }

    #[test]
    fn schedule_try_from_errors_on_unknown_in_multiple() {
        assert!(Schedule::try_from(vec!["MONDAY", "FUNDAY"]).is_err());
    }
}
