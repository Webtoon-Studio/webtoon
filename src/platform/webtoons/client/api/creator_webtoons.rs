use serde::Deserialize;

use crate::platform::webtoons::Type;

#[derive(Deserialize)]
pub struct CreatorWebtoons {
    pub result: Result,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub titles: Vec<Titles>,
    pub total_count: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Titles {
    pub id: u32,
    #[serde(rename = "subject")]
    pub title: String,
    pub authors: Vec<Authors>,
    pub genres: Vec<String>,
    #[serde(rename = "grade")]
    pub r#type: Type,
    pub thumbnail_url: String,
    pub recent_episode_registered_at: i64,
    pub title_registered_at: i64,
}

#[derive(Deserialize)]
pub struct Authors {
    pub nickname: String,
}
