// https://comic.naver.com/api/article/list/info?titleId=183559

use serde::Deserialize;

use crate::platform::naver::{
    meta::{Genre, Release},
    Type,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
pub(in crate::platform::naver) struct Info {
    #[serde(rename = "titleId")]
    pub id: u64,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail: String,
    #[serde(rename = "titleName")]
    pub title: String,
    #[serde(rename = "webtoonLevelCode")]
    pub r#type: Type,
    #[serde(rename = "rest")]
    pub on_hiatus: bool,
    #[serde(rename = "finished")]
    pub is_completed: bool,
    #[serde(rename = "publishDayOfWeekList")]
    pub weekdays: Vec<Release>,
    #[serde(rename = "communityArtists")]
    pub creators: Vec<Creator>,
    #[serde(rename = "synopsis")]
    pub summary: String,
    #[serde(rename = "favorite")]
    pub is_favorited: bool,
    #[serde(rename = "favoriteCount")]
    pub favorites: u32,
    // #[serde(rename = "firstArticle")]
    // pub first_episode: FirstEpisode,
    #[serde(rename = "gfpAdCustomParam")]
    pub data: Data,
    #[serde(rename = "new")]
    pub is_new: bool,
}

#[derive(Debug, Deserialize)]
pub(in crate::platform::naver) struct Creator {
    #[serde(rename = "artistId")]
    pub id: u64,
    #[serde(rename = "name")]
    pub username: String,
    // "artistTypeList": [
    //     "ARTIST_WRITER",
    //     "ARTIST_PAINTER"
    // ],
    #[serde(alias = "curationPageUrl", alias = "profilePageUrl")]
    pub page: String, // "https://comic.naver.com/artistTitle?id=155779"
}

#[derive(Debug, Deserialize)]
pub(in crate::platform::naver) struct FirstEpisode {
    pub no: u16,
    #[serde(rename = "firstArticle")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub(in crate::platform::naver) struct Data {
    #[serde(rename = "webtoonLevelCode")]
    pub r#type: Type,
    #[serde(rename = "titleName")]
    pub title: String,
    #[serde(rename = "displayAuthor")]
    pub author: String,
    #[serde(rename = "rankGenreTypes")]
    pub genres: Vec<Genre>,
    #[serde(rename = "weekdays")]
    pub weekdays: Vec<Release>,
}
