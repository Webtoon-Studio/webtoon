//! Contains metadata implementations for webtoons.com.

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;

/// An error that can occur when parsing a language from a URL path.
#[derive(Debug, Error)]
#[error(
    "failed to parse `{0}` into `Language` should be one of `en`, `zh-hant`, `th`, `id`, `de`, `es`, `fr`"
)]
pub struct ParseLanguageError(String);

/// Represents the languages that `webtoons.com` has.
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Language {
    /// English
    #[default]
    En,
    /// Chinese
    Zh,
    /// Thai
    Th,
    /// Indonesian
    Id,
    /// Spanish
    Es,
    /// French
    Fr,
    /// German
    De,
}

impl FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            "zh-hant" => Ok(Self::Zh),
            "th" => Ok(Self::Th),
            "id" => Ok(Self::Id),
            "es" => Ok(Self::Es),
            "fr" => Ok(Self::Fr),
            "de" => Ok(Self::De),
            _ => Err(ParseLanguageError(s.to_owned())),
        }
    }
}

/// Represents the type a webtoon can be on webtoons.com.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// An Original webtoon.
    #[serde(alias = "WEBTOON")]
    Original,
    /// A Canvas webtoon.
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

/// An Error that can occur when parsing a letter to a [`Type`].
///
/// Only `w` and `c` are valid.
#[derive(Debug, Error)]
pub enum ParseLetterError {
    /// An invalid letter was found
    #[error("`{0}` is an invalid letter, should only be `w` or `c`")]
    InvalidLetter(String),
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

    /// "c" or "w"
    pub(crate) fn as_single_letter(self) -> &'static str {
        match self {
            Self::Canvas => "c",
            Self::Original(_) => "w",
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

/// Represents a genre on the webtoons.com platform.
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

    // Doing only official ones here. Custom will be done at the source.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "COMEDY" | "Comedy" | "comedy" | "ตลก" | "komedi" | "Comedia" | "Comédie" => {
                Ok(Self::Comedy)
            }
            "FANTASY"
            | "Fantasy"
            | "fantasy"
            | "奇幻冒險"
            | "แฟนตาซี"
            | "fantasi"
            | "Fantasía"
            | "Fantastique" => Ok(Self::Fantasy),
            "ROMANCE" | "Romance" | "romance" | "愛情" | "โรแมนซ์" | "romantis" | "Romantisch" => {
                Ok(Self::Romance)
            }
            "SLICE OF LIFE"
            | "Slice of life"
            | "slice-of-life"
            | "搞笑/生活"
            | "ชีวิตประจำวัน"
            | "Vida cotidiana"
            | "Tranche de vie"
            | "Alltagsstory" => Ok(Self::SliceOfLife),
            "SCI-FI" | "Sci-fi" | "Sci-Fi" | "sf" | "SF" | "科幻" | "ไซไฟ" | "fiksi ilmiah"
            | "Ciencia ficción" => Ok(Self::SciFi),
            "DRAMA" | "Drama" | "drama" | "劇情" | "ดราม่า" => Ok(Self::Drama),
            "SHORT STORY" | "Short story" => Ok(Self::ShortStory),
            "ACTION" | "Action" | "action" | "動作" | "แอกชัน" | "aksi" | "Acción" => {
                Ok(Self::Action)
            }
            "ALL AGES" | "All Ages" => Ok(Self::AllAges),
            "SUPERHERO"
            | "Superhero"
            | "super-hero"
            | "superhero"
            | "超級英雄"
            | "ซูเปอร์ฮีโร่"
            | "Superhéroes"
            | "Superhéros"
            | "Superhelden" => Ok(Self::Superhero),
            "HEARTWARMING"
            | "Heartwarming"
            | "heartwarming"
            | "療癒/萌系"
            | "อบอุ่นหัวใจ"
            | "menyentuh"
            | "Conmovedor" => Ok(Self::Heartwarming),
            "THRILLER" | "Thriller" | "thriller" | "驚悚/恐怖" | "ระทึกขวัญ" | "Suspenso" => {
                Ok(Self::Thriller)
            }
            "HORROR" | "Horror" | "horror" | "สยองขวัญ" | "horor" | "Terror" | "Horreur" => {
                Ok(Self::Horror)
            }
            "POST APOCALYPTIC" | "Post apocalyptic" | "Post-apocalyptic" => {
                Ok(Self::PostApocalyptic)
            }
            "ZOMBIES" | "Zombies" | "zombies" => Ok(Self::Zombies),
            "SCHOOL" | "School" | "school" | "校園" => Ok(Self::School),
            "SUPERNATURAL" | "Supernatural" | "supernatural" | "PARANORMAL" | "Paranormal" => {
                Ok(Self::Supernatural)
            }
            "ANIMALS" | "Animals" | "animals" => Ok(Self::Animals),
            "CRIME/MYSTERY" | "Crime/Mystery" | "Mystery" | "mystery" | "懸疑推理" => {
                Ok(Self::Mystery)
            }
            "HISTORICAL"
            | "Historical"
            | "historical"
            | "古裝"
            | "ย้อนยุค"
            | "sejarah"
            | "Histórico" => Ok(Self::Historical),
            "INFORMATIVE" | "Informative" | "informative" | "tiptoon" | "生活常識漫畫" | "ทิปตูน"
            | "tips & trik" | "Informativo" | "Info" => Ok(Self::Informative),
            "SPORTS" | "Sports" | "sports" | "運動" | "กีฬา" | "olahraga" | "Deportes" => {
                Ok(Self::Sports)
            }
            "INSPIRATIONAL" | "Inspirational" | "inspirational" => Ok(Self::Inspirational),
            "LGBTQ+ / Y" | "LGBTQ+" | "bl-gl" => Ok(Self::LGBTQ),
            "romantic-fantasy" | "โรแมนซ์แฟนตาซี" | "kerajaan" => {
                Ok(Self::RomanticFantasy)
            }
            "martial-arts" | "武俠" => Ok(Self::MartialArts),
            "western-palace" | "歐式宮廷" => Ok(Self::WesternPalace),
            "eastern-palace" | "古代宮廷" => Ok(Self::EasternPalace),
            "romance-m" | "大人系" => Ok(Self::MatureRomance),
            "time-slip" | "穿越/轉生" => Ok(Self::TimeSlip),
            "local" | "台灣原創作品" | "LOKAL" => Ok(Self::Local),
            "city-office" | "現代/職場" => Ok(Self::CityOffice),
            "adaptation" | "影視化" => Ok(Self::Adaptation),
            "shonen" | "少年" => Ok(Self::Shonen),
            "web-novel" | "WEBNOVEL" | "小說" | "นิยาย" => Ok(Self::WebNovel),
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
