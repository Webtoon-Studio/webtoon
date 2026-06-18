//! Contains metadata implementations for `webtoons.com`.

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;

/// Represents the type a Webtoon can be on `webtoons.com`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// An Original Webtoon.
    #[serde(alias = "WEBTOON")]
    Original,
    /// A Canvas Webtoon.
    #[serde(alias = "CHALLENGE")]
    Canvas,
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WEBTOON" => Ok(Self::Original),
            "CHALLENGE" => Ok(Self::Canvas),
            _ => {
                bail!("`{s}` is not a valid webtoon type. Expected one of `WEBTOON` or `CHALLENGE`")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Scope {
    Original(Genre),
    Canvas,
}

impl Scope {
    pub(super) fn as_slug(&self) -> &str {
        match self {
            Self::Canvas => "canvas",
            Self::Original(genre) => genre.as_slug(),
        }
    }
}

impl FromStr for Scope {
    type Err = ParseGenreError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let scope = match s {
            "canvas" => Self::Canvas,
            slug => Self::Original(Genre::from_str(slug)?),
        };

        Ok(scope)
    }
}

/// Represents a genre on `webtoons.com`.
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
            Self::SciFi => "sf",
            Self::ShortStory => "short-story",
            Self::SliceOfLife => "slice-of-life",
            Self::Sports => "sports",
            Self::Superhero => "super-hero",
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
            "SUPERNATURAL" | "Supernatural" | "supernatural" => Ok(Self::Supernatural),
            "ANIMALS" | "Animals" | "animals" => Ok(Self::Animals),
            "CRIME/MYSTERY" | "Crime/Mystery" | "Mystery" | "mystery" => Ok(Self::Mystery),
            "HISTORICAL" | "Historical" | "historical" => Ok(Self::Historical),
            "INFORMATIVE" | "Informative" | "informative" | "tiptoon" => Ok(Self::Informative),
            "SPORTS" | "Sports" | "sports" => Ok(Self::Sports),
            "INSPIRATIONAL" | "Inspirational" | "inspirational" => Ok(Self::Inspirational),
            "LGBTQ+ / Y" | "LGBTQ+" | "bl-gl" | "LGBTQI+" => Ok(Self::LGBTQ),
            "romantic-fantasy" | "Romance Fantasy" | "ROMANTIC_FANTASY" => {
                Ok(Self::RomanticFantasy)
            }
            "martial-arts" => Ok(Self::MartialArts),
            "western-palace" => Ok(Self::WesternPalace),
            "eastern-palace" => Ok(Self::EasternPalace),
            "romance-m" => Ok(Self::MatureRomance),
            "time-slip" => Ok(Self::TimeSlip),
            "local" | "LOCAL" => Ok(Self::Local),
            "city-office" => Ok(Self::CityOffice),
            "adaptation" => Ok(Self::Adaptation),
            "shonen" => Ok(Self::Shonen),
            "web-novel" | "WEBNOVEL" => Ok(Self::WebNovel),
            "graphic-novel" | "GRAPHIC_NOVEL" | "Graphic Novel" => Ok(Self::GraphicNovel),
            _ => Err(ParseGenreError(s.to_owned())),
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
