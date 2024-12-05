use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct UserAction {
    pub favorite_info: FavoriteInfo,
    pub manager: bool,
    pub star_info: StarInfo,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct FavoriteInfo {
    pub favorite: bool,
    pub favorite_count: i64,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct StarInfo {
    pub average_star_score: f64,
    pub star: bool,
    pub star_score: u32,
    #[serde(rename = "starScoreCount")]
    pub scorers: u32,
}
