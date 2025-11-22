//! Module for webtoons.com search API.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RawSearch {
    pub result: SearchResult,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub challenge_title_list: Option<Canvas>,
    pub webtoon_title_list: Option<Originals>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Originals {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub content_id: u32,
    pub content_sub_type: String,
    pub extra: Extra,
    pub name: String,
    pub service_type: String,
    pub thumbnail: Thumbnail,
}

#[derive(Serialize, Deserialize)]
pub struct Illustrator {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct Writer {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct Thumbnail {
    pub domain: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub illustrator: Illustrator,
    pub unsuitable_for_children: Option<bool>,
    pub writer: Writer,
}
