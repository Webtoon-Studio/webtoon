use anyhow::{Context, anyhow};
// use chrono::serde::ts_milliseconds_option;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::platform::webtoons::{dashboard::episodes::DashboardStatus, errors::EpisodeError};

pub fn parse(html: &str) -> Result<Vec<DashboardEpisode>, EpisodeError> {
    // PERF: Creating new string during cleaning
    fn clean(line: &str) -> String {
        // Removes `dashboardEpisodeList: ` from the front and `,` from the back
        let cleaned = line[22..line.len() - 1]
            .replace(r"\'", "'")
            .replace(r"\x3c", "<")
            // If left in, the HTML entity decode will leave raw `"` in the string, leaving a malformed JSON because of it
            .replace("&quot;", r#"\""#);

        html_escape::decode_html_entities(&cleaned).to_string()
    }

    for line in html.lines().rev() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("dashboardEpisodeList") {
            return Ok(
                serde_json::from_str::<Vec<DashboardEpisode>>(&clean(trimmed))
                    .with_context(|| trimmed.to_string())?,
            );
        }
    }

    Err(EpisodeError::Unexpected(anyhow!(
        "failed to find `dashboardEpisodeList` as the start of any line:\n\n{html}"
    )))
}

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
