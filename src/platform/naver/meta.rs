//! Contains metadata implementations for `comic.naver.com`.

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;

/// Represents the type a webtoon can be on `comic.naver.com`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// A Featured Webtoon.
    Featured,
    /// A Best Challenge Webtoon.
    BestChallenge,
    /// A Challenge Webtoon.
    Challenge,
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WEBTOON" => Ok(Self::Featured),
            "BEST_CHALLENGE" => Ok(Self::BestChallenge),
            "CHALLENGE" => Ok(Self::Challenge),
            _ => {
                bail!(
                    "`{s}` is not a valid webtoon type. Expected one of `WEBTOON`, `BEST_CHALLENGE` or `CHALLENGE`"
                )
            }
        }
    }
}

/// Represents a genre on the `comic.naver.com` platform.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
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
    GraphicNovel,
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
            Self::GraphicNovel => "graphic-novel",
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
            "COMEDY" | "Comedy" | "comedy" => Ok(Self::Comedy),
            "FANTASY" | "Fantasy" | "fantasy" => Ok(Self::Fantasy),
            "ROMANCE" | "Romance" | "romance" => Ok(Self::Romance),
            "SLICE OF LIFE" | "Slice of life" | "slice-of-life" => Ok(Self::SliceOfLife),
            "SCI-FI" | "Sci-fi" | "Sci-Fi" | "sf" | "SF" => Ok(Self::SciFi),
            "DRAMA" | "Drama" | "drama" => Ok(Self::Drama),
            "SHORT STORY" | "Short story" => Ok(Self::ShortStory),
            "ACTION" | "Action" | "action" => Ok(Self::Action),
            "ALL AGES" | "All Ages" => Ok(Self::AllAges),
            "SUPERHERO" | "Superhero" | "super-hero" | "superhero" => Ok(Self::Superhero),
            "HEARTWARMING" | "Heartwarming" | "heartwarming" => Ok(Self::Heartwarming),
            "THRILLER" | "Thriller" | "thriller" => Ok(Self::Thriller),
            "HORROR" | "Horror" | "horror" => Ok(Self::Horror),
            "POST APOCALYPTIC" | "Post apocalyptic" | "Post-apocalyptic" => {
                Ok(Self::PostApocalyptic)
            }
            "ZOMBIES" | "Zombies" | "zombies" => Ok(Self::Zombies),
            "SCHOOL" | "School" | "school" => Ok(Self::School),
            "SUPERNATURAL" | "Supernatural" | "supernatural" | "PARANORMAL" | "Paranormal" => {
                Ok(Self::Supernatural)
            }
            "ANIMALS" | "Animals" | "animals" => Ok(Self::Animals),
            "CRIME/MYSTERY" | "Crime/Mystery" | "Mystery" | "mystery" => Ok(Self::Mystery),
            "HISTORICAL" | "Historical" | "historical" => Ok(Self::Historical),
            "INFORMATIVE" | "Informative" | "informative" | "tiptoon" | "Info" => {
                Ok(Self::Informative)
            }
            "SPORTS" | "Sports" | "sports" => Ok(Self::Sports),
            "INSPIRATIONAL" | "Inspirational" | "inspirational" => Ok(Self::Inspirational),
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
            "web-novel" | "WEBNOVEL" => Ok(Self::WebNovel),
            "graphic-novel" | "GRAPHIC_NOVEL" => Ok(Self::GraphicNovel),
            _ => Err(ParseGenreError(s.to_owned())),
        }
    }
}

/// Represents a day of the week.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

/// An error that can happen when parsing a string into a [`Weekday`].
#[derive(Debug, Error)]
#[error("failed to parse `{0}` into a weekday")]
pub struct ParseWeekdayError(String);

impl FromStr for Weekday {
    type Err = ParseWeekdayError;

    fn from_str(weekday: &str) -> Result<Self, Self::Err> {
        match weekday {
            "SUNDAY" => Ok(Self::Sunday),
            "MONDAY" => Ok(Self::Monday),
            "TUESDAY" => Ok(Self::Tuesday),
            "WEDNESDAY" => Ok(Self::Wednesday),
            "THURSDAY" => Ok(Self::Thursday),
            "FRIDAY" => Ok(Self::Friday),
            "SATURDAY" => Ok(Self::Saturday),
            _ => Err(ParseWeekdayError(weekday.to_owned())),
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
