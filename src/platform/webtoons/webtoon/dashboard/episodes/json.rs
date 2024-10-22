use anyhow::{anyhow, Context};
// use chrono::serde::ts_milliseconds_option;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::hash::Hash;
use std::str::FromStr;
use thiserror::Error;

use crate::platform::webtoons::errors::EpisodeError;
use crate::platform::webtoons::webtoon::episode::AdStatus;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct DashboardEpisode {
    #[serde(alias = "episode")]
    pub metadata: Metadata,

    #[serde(default)]
    #[serde(alias = "exposureDate")]
    #[serde(alias = "freeExposeOrReservationDate")]
    pub published: Option<i64>,

    // #[serde(default)]
    // #[serde(alias = "rewardAdOnDate")]
    // pub reward_ad_on_date: Option<RewardAdOnDate>,

    // #[serde(default)]
    // #[serde(alias = "rewardAdOffDate")]
    // pub reward_ad_off_date: Option<RewardAdOffDate>,

    // PUBLISHED
    // DRAFT
    // READY
    // AD_ON
    // AD_OFF
    // REMOVED
    // APPROVED
    // IN_REVIEW
    // DISAPPROVED
    // DISAPPROVED_AUTO
    #[serde(alias = "dashboardStatus")]
    pub dashboard_status: DashboardStatus,

    #[serde(alias = "commentActive")]
    pub comment_exposure: bool,
}

impl DashboardEpisode {
    pub(super) fn parse(html: &str) -> Result<Vec<Self>, EpisodeError> {
        for line in html.lines().rev() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("dashboardEpisodeList") {
                return Ok(serde_json::from_str::<Vec<Self>>(&clean(trimmed))
                    .with_context(|| trimmed.to_string())?);
            }
        }

        Err(EpisodeError::Unexpected(anyhow!(
            "failed to find `dashboardEpisodeList` as the start of any line:\n\n{html}"
        )))
    }
}

// PERF: Creating new string during cleaning
fn clean(line: &str) -> String {
    // removes `dashboardEpisodeList: ` from the front and `,` from the back
    let cleaned = line[22..line.len() - 1]
        .replace(r"\'", "'")
        .replace(r"\x3c", "<")
        // If left in, the html entity decode will leave raw `"` in the string, leaving a malformed json because of it
        .replace("&quot;", r#"\""#);

    html_escape::decode_html_entities(&cleaned).to_string()
}

#[derive(DeserializeFromStr, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardStatus {
    Published,
    Draft,
    Approved,
    Removed,
    Ready,
    AdOn,
    AdOff,
    InReview,
    Disapproved,
    DisapprovedAuto,
}

impl DashboardStatus {
    #[allow(dead_code)]
    pub fn is_published(self) -> bool {
        matches!(self, Self::Published | Self::AdOn | Self::AdOff)
    }

    pub fn ad_status(self) -> AdStatus {
        match self {
            Self::Published
            | Self::Draft
            | Self::Ready
            | Self::Approved
            | Self::Removed
            | Self::InReview
            | Self::Disapproved
            | Self::DisapprovedAuto => AdStatus::Never,
            Self::AdOn => AdStatus::Yes,
            Self::AdOff => AdStatus::No,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("failed to parse `{0}` into a `DashboardStatus` expected one of PUBLISHED, READY, DRAFT, IN_REVIEW, APPROVED, REMOVED, AD_ON, or AD_OFF")]
pub struct DashboardStatusParseError(String);

impl FromStr for DashboardStatus {
    type Err = DashboardStatusParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PUBLISHED" => Ok(Self::Published),
            "DRAFT" => Ok(Self::Draft),
            "READY" => Ok(Self::Ready),
            "AD_ON" => Ok(Self::AdOn),
            "AD_OFF" => Ok(Self::AdOff),
            "REMOVED" => Ok(Self::Removed),
            "APPROVED" => Ok(Self::Approved),
            "IN_REVIEW" => Ok(Self::InReview),
            "DISAPPROVED" => Ok(Self::Disapproved),
            "DISAPPROVED_AUTO" => Ok(Self::DisapprovedAuto),
            unknown => Err(DashboardStatusParseError(unknown.to_string())),
        }
    }
}

// #[derive(Debug, Deserialize, Clone, Copy, Serialize, Hash, PartialEq, Eq)]
// pub struct RewardAdOnDate {
//     #[serde(default)]
//     #[serde(alias = "dateTime")]
//     #[serde(deserialize_with = "deserialize_datetime_utc_from_milliseconds_optional")]
//     pub reward_published: Option<DateTime<Utc>>,
// }

// TODO: Sometimes it doesnt always return a timestamp
// "rewardAdOffDate":{"status":"NOT_YET_PUBLISHED"}
// #[derive(Debug, Deserialize, Clone, Serialize, Hash, PartialEq, Eq)]
// pub struct RewardAdOffDate {
// #[serde(alias = "dateTime")]
// #[serde(with = "ts_milliseconds_option")]
// pub reward_published: Option<DateTime<Utc>>,
// // NOT_YET_PUBLISHED
// pub status: Option<String>,
// }

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Metadata {
    #[serde(alias = "episodeNo")]
    pub number: u16,

    #[serde(alias = "episodeTitle")]
    pub title: String,

    #[serde(alias = "exposed")]
    pub is_published: bool,

    #[serde(alias = "readCount")]
    pub views: u32,

    #[serde(alias = "likeitCount")]
    pub likes: u32,

    // NOTE: DRAFT episodes dont have a `thumbnailImageUrl` field.
    #[serde(alias = "thumbnailImageUrl")]
    pub thumbnail: Option<String>,

    #[serde(skip_deserializing)]
    #[serde(alias = "creatorNote")]
    pub creator_note: String,
}

impl Hash for DashboardEpisode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.metadata.number.hash(state);
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.hash(state);
        }
    }
}
