//! Module for webtoons.com search API.
#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawSearch {
    pub result: SearchResult,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub challenge_title_list: Option<Canvas>,
    pub webtoon_title_list: Option<Originals>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Originals {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub data: Vec<Data>,
    pub pagination: Pagination,
    pub total_count: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(deserialize_with = "crate::stdx::serde::u32_from_string")]
    pub content_id: u32,
    pub content_sub_type: String,
    pub extra: Extra,
    pub name: String,
    pub service_type: String,
    pub thumbnail: Thumbnail,
}

#[derive(Deserialize)]
pub struct Illustrator {
    pub nickname: String,
}

#[derive(Deserialize)]
pub struct Writer {
    pub nickname: String,
}

#[derive(Deserialize)]
pub struct Thumbnail {
    pub domain: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub next: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub illustrator: Illustrator,
    pub unsuitable_for_children: Option<bool>,
    pub writer: Writer,
}
