use serde::Deserialize;
use std::hash::Hash;

use crate::{
    platform::webtoons::{dashboard::episodes::DashboardStatus, error::EpisodeError},
    stdx::error::invariant,
};

pub fn parse(html: &str) -> Result<Vec<DashboardEpisode>, EpisodeError> {
    const START: &str = "dashboardEpisodeList: ";
    const END: char = ',';

    fn clean(line: &str) -> String {
        // Removes `dashboardEpisodeList: ` from the front and `,` from the back
        let cleaned = line
            .trim_start_matches(START)
            .trim_end_matches(END)
            .replace(r"\'", "'")
            .replace(r"\x3c", "<")
            // If left in, the HTML entity decode will leave raw `"` in the string, leaving a malformed JSON because of it
            .replace("&quot;", r#"\""#);

        html_escape::decode_html_entities(&cleaned).to_string()
    }

    if let Some(json) = html
        .lines()
        // PERF:
        // The wanted line is closer to the bottom
        // of the HTML so start there and work up.
        .rev()
        .map(|line| line.trim_start())
        .find(|line| line.starts_with(START) && line.ends_with(END))
        .map(|line| clean(line))
    {
        match serde_json::from_str::<Vec<DashboardEpisode>>(&json) {
            Ok(episodes) => return Ok(episodes),
            Err(err) => invariant!(
                "failed to deserialize `dashboardEpisodeList` json from `webtoons.com` episode dashboard: {err}\n\n{json}"
            ),
        }
    }

    invariant!(
        "failed to find line that starts with `{START}` and ends with `{END}` on `webtoons.com` episode dashboard:\n\n{html}"
    );
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
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
        for episode in data {
            episode.hash(state);
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
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

    // NOTE: DRAFT episodes don't have a `thumbnailImageUrl` field.
    #[serde(alias = "thumbnailImageUrl")]
    pub thumbnail: Option<String>,

    #[serde(skip_deserializing)]
    #[serde(alias = "creatorNote")]
    pub creator_note: String,
}

// use chrono::serde::ts_milliseconds_option;

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
