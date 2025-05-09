use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    pub star_info: StarInfo,
    // NOTE: always `0`, so not helpful.
    // view_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct StarInfo {
    pub star: bool,
    pub star_score: u32,
    pub average_star_score: f64,
    pub star_score_count: u32,
}
