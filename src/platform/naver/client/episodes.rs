pub(super) mod rating;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct Response {
    pub article_list: Vec<FreeEpisode>,
    pub charge_folder_article_list: Vec<PaidEpisode>,
    #[serde(rename = "finished")]
    pub is_finished: bool,
    pub page_info: PageInfo,
    #[serde(rename = "totalCount")]
    pub episodes: u16,
}

#[derive(Deserialize)]
pub(in crate::platform::naver) struct FreeEpisode {
    pub no: u16,
    #[serde(rename = "serviceDateDescription")]
    pub date: String,
    #[serde(rename = "starScore")]
    pub rating: f64,
    #[serde(rename = "subtitle")]
    pub title: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail: String,
}

#[derive(Deserialize)]
pub(in crate::platform::naver) struct PaidEpisode {
    pub no: u16,
    #[serde(rename = "starScore")]
    pub rating: f64,
    #[serde(rename = "subtitle")]
    pub title: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct PageInfo {
    pub total_pages: u8,
}
