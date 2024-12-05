//! Contains metadata implementations for webtoons.com.

use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use thiserror::Error;

/// Represents the type a webtoon can be on `comic.naver.com`.
#[derive(
    Debug, Clone, Copy, DeserializeFromStr, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Type {
    /// An Original webtoon.
    Original,
    /// A Canvas webtoon.
    Canvas,
}

impl Type {
    pub(super) fn as_ticket(&self) -> &'static str {
        match self {
            Type::Original => "comic",
            Type::Canvas => "comic_challenge",
        }
    }

    pub(super) fn as_slug(&self) -> &'static str {
        match self {
            Type::Original => "webtoon",
            Type::Canvas => "challenge",
        }
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WEBTOON" => Ok(Self::Original),
            "CHALLENGE" | "BEST_CHALLENGE" => Ok(Self::Canvas),
            _ => anyhow::bail!("failed to parse into `Type`, expected either `WEBTOON` or `CHALLENGE` but got `{s}`"),
        }
    }
}

/// Represents a genre on the webtoons.com platform.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, DeserializeFromStr, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash,
)]
pub enum Genre {
    Comedy,
    Fantasy,
    Romance,
    SliceOfLife,
    SciFi,
    Drama,
    ShortStory,
    Action,
    Superhero,
    Heartwarming,
    Thriller,
    Horror,
    PostApocalyptic,
    Zombies,
    School,
    Supernatural,
    Animals,
    Mystery,
    Historical,
    /// Tiptoon
    Informative,
    Sports,
    Inspirational,
    AllAges,
    LGBTQ,
    RomanticFantasy,
    MartialArts,
    WesternPalace,
    EasternPalace,
    MatureRomance,
    /// Reincarnation/Time-travel
    TimeSlip,
    Local,
    /// Modern/Workplace
    CityOffice,
    Adaptation,
    Shonen,
    WebNovel,
}

impl Genre {
    /// Converts a [`Genre`] into a URL safe slug.
    ///
    /// Example:
    /// - `Genre::Action => "action"`,
    /// - `Genre::AllAges => "all-ages"`,
    #[inline]
    #[must_use]
    pub const fn as_slug(&self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::AllAges => "all-ages",
            Self::Animals => "animals",
            Self::Comedy => "comedy",
            Self::Drama => "drama",
            Self::Fantasy => "fantasy",
            Self::Heartwarming => "heartwarming",
            Self::Historical => "historical",
            Self::Horror => "horror",
            Self::Informative => "tiptoon",
            Self::Inspirational => "inspirational",
            Self::Mystery => "mystery",
            Self::PostApocalyptic => "post-apocalyptic",
            Self::Romance => "romance",
            Self::School => "school",
            Self::SciFi => "sci-fi",
            Self::ShortStory => "short-story",
            Self::SliceOfLife => "slice-of-life",
            Self::Sports => "sports",
            Self::Superhero => "superhero",
            Self::Supernatural => "supernatural",
            Self::Thriller => "thriller",
            Self::Zombies => "zombies",
            Self::LGBTQ => "bl-gl",
            Self::RomanticFantasy => "romantic-fantasy",
            Self::MartialArts => "martial-arts",
            Self::WesternPalace => "western-palace",
            Self::EasternPalace => "eastern-palace",
            Self::MatureRomance => "romance-m",
            Self::TimeSlip => "time-slip",
            Self::Local => "local",
            Self::CityOffice => "city-office",
            Self::Adaptation => "adaptation",
            Self::Shonen => "shonen",
            Self::WebNovel => "web-novel",
        }
    }
}

/// An error that can happen when parsing a string into a [`Genre`].
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a known genre")]
pub struct ParseGenreError(String);

impl FromStr for Genre {
    type Err = ParseGenreError;

    // Doing only official ones here. Custom will be done at the source.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "COMEDY" | "HUMOR" => Ok(Self::Comedy),
            "FANTASY" => Ok(Self::Fantasy),
            "ROMANCE" => Ok(Self::Romance),
            "SLICE OF LIFE" => Ok(Self::SliceOfLife),
            "SCI-FI" => Ok(Self::SciFi),
            "DRAMA" => Ok(Self::Drama),
            "SHORT STORY" => Ok(Self::ShortStory),
            "ACTION" => Ok(Self::Action),
            "ALL AGES" => Ok(Self::AllAges),
            "SUPERHERO" => Ok(Self::Superhero),
            "HEARTWARMING" => Ok(Self::Heartwarming),
            "THRILLER" => Ok(Self::Thriller),
            "HORROR" => Ok(Self::Horror),
            "POST APOCALYPTIC" => Ok(Self::PostApocalyptic),
            "ZOMBIES" => Ok(Self::Zombies),
            "SCHOOL" => Ok(Self::School),
            "SUPERNATURAL" => Ok(Self::Supernatural),
            "ANIMALS" => Ok(Self::Animals),
            "CRIME/MYSTERY" => Ok(Self::Mystery),
            "HISTORICAL" => Ok(Self::Historical),
            "INFORMATIVE" => Ok(Self::Informative),
            "SPORTS" => Ok(Self::Sports),
            "INSPIRATIONAL" => Ok(Self::Inspirational),
            "LGBTQ+ / Y" | "LGBTQ+" | "bl-gl" => Ok(Self::LGBTQ),
            "romantic-fantasy" => Ok(Self::RomanticFantasy),
            "martial-arts" => Ok(Self::MartialArts),
            "western-palace" => Ok(Self::WesternPalace),
            "eastern-palace" => Ok(Self::EasternPalace),
            "romance-m" => Ok(Self::MatureRomance),
            "time-slip" => Ok(Self::TimeSlip),
            "local" => Ok(Self::Local),
            "city-office" => Ok(Self::CityOffice),
            "adaptation" => Ok(Self::Adaptation),
            "shonen" => Ok(Self::Shonen),
            "web-novel" => Ok(Self::WebNovel),
            _ => Err(ParseGenreError(s.to_owned())),
        }
    }
}

/// Represents a kind of release schedule for Originals.  
///
/// For the days of the week, a webtoon can have multiple.
///
/// If its not a day of the week, it can only be either `Daily` or `Completed`, alone.
#[derive(Debug, Clone, Copy, DeserializeFromStr, Serialize, PartialEq, Eq, PartialOrd, Ord)]
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
}

/// An error which can happen when parsing a string to a [`Release`].
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a `Release`")]
pub struct ParseReleaseError(String);

impl FromStr for Release {
    type Err = ParseReleaseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "월" | "MONDAY" => Ok(Self::Monday),
            "화" | "TUESDAY" => Ok(Self::Tuesday),
            "수" | "WEDNESDAY" => Ok(Self::Wednesday),
            "목" | "THURSDAY" => Ok(Self::Thursday),
            "금" | "FRIDAY" => Ok(Self::Friday),
            "토" | "SATURDAY" => Ok(Self::Saturday),
            "일" | "SUNDAY" => Ok(Self::Sunday),
            _ => Err(ParseReleaseError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_genres_from_str() -> Result<(), Box<dyn std::error::Error>> {
        {
            let genre = Genre::from_str("Slice of life")?;

            pretty_assertions::assert_eq!(Genre::SliceOfLife, genre);

            Ok(())
        }
    }
}
