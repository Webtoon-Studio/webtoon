use serde::{Deserialize, Serialize};

use crate::platform::naver::meta::Weekday;

#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Info {
    pub ad_banner_list: Vec<serde_json::Value>,
    pub age: Age,
    pub charge_best_challenge: bool,
    pub community_artists: Vec<CommunityArtists>,
    // pub contents_no: i64,
    // pub curation_tag_list: Vec<Tag>,
    pub daily_pass: bool,
    pub except_bm_info: bool,
    pub favorite: bool,
    pub favorite_count: u32,
    pub finished: bool,
    pub first_article: FirstArticle,
    pub gfp_ad_custom_param: GfpAdCustomParam,
    pub new: bool,
    pub poster_thumbnail_url: String,
    pub publish_day_of_week_list: Vec<Weekday>,
    // pub publish_description: String,
    pub rest: bool,
    pub shared_thumbnail_url: String,
    pub synopsis: String,
    pub thumbnail_badge_list: Vec<String>,
    pub thumbnail_url: String,
    pub title_id: i64,
    pub title_name: String,
    pub webtoon_level_code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GfpAdCustomParam {
    pub adult_yn: String,
    // pub cid: i64,
    // pub cp_name: String,
    // pub cpid: String,
    pub daily_free_yn: String,
    pub daily_plus_yn: String,
    pub display_author: String,
    pub finished_yn: String,
    pub genre_types: Vec<String>,
    pub rank_genre_types: Vec<String>,
    pub tags: Vec<String>,
    pub title_id: i64,
    pub title_name: String,
    pub webtoon_level_code: String,
    pub weekdays: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FirstArticle {
    pub charge: bool,
    pub no: i64,
    pub subtitle: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Tag {
    pub curation_type: String,
    pub id: i64,
    pub tag_name: String,
    pub url_path: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CommunityArtists {
    pub artist_id: i64,
    pub artist_type_list: Vec<String>,
    pub name: String,

    pub post_description: Option<String>,
    pub profile_badge: Option<String>,
    pub profile_image_url: Option<String>,
    pub profile_page_scheme: Option<String>,
    pub profile_page_url: Option<String>,
    pub curation_page_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Age {
    pub description: String,
    pub r#type: String,
}

// "adBannerList": [
//         {
//             "bannerType": "GAME",
//             "imageUrl": "https://naverwebtoon-phinf.pstatic.net/20250428_57/17458262197953nAg4_JPEG/upload_14858518398799611966.JPEG",
//             "targetUrl": "https://play.google.com/store/apps/details?id=com.studiolico.mythicitem",
//             "statViewUrl": "/api/stat/gameBanner?type=VIEW&target=ARTICLE_LIST&bannerId=1435",
//             "statClickUrl": "/api/stat/gameBanner?type=CLICK&target=ARTICLE_LIST&bannerId=1435",
//             "imageHeight": 52,
//             "imageWidth": 840
//         },
//         {
//             "bannerType": "GAME",
//             "imageUrl": "https://naverwebtoon-phinf.pstatic.net/20250428_292/1745826247239EVVkF_JPEG/upload_18242950960447727900.JPEG",
//             "targetUrl": "https://play.google.com/store/apps/details?id=com.studiolico.mythicitem",
//             "statViewUrl": "/api/stat/gameBanner?type=VIEW&target=ARTICLE_LIST&bannerId=1436",
//             "statClickUrl": "/api/stat/gameBanner?type=CLICK&target=ARTICLE_LIST&bannerId=1436",
//             "imageHeight": 52,
//             "imageWidth": 840
//         }
//     ],
