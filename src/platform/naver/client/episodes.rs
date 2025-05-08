use serde::Deserialize;
use std::fmt::Display;

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub article_list: Vec<Article>,
    pub charge_folder_article_list: Vec<Article>,
    pub page_info: PageInfo,
    pub total_count: i64,
}

#[allow(unused, clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub bgm: bool,
    pub charge: bool,
    pub has_read_log: bool,
    pub no: u16,
    pub recently_read_log: bool,
    pub service_date_description: String,
    pub star_score: f64,
    pub subtitle: String,
    pub thumbnail_clock: bool,
    pub thumbnail_lock: bool,
    pub thumbnail_url: String,
    pub up: bool,
    // The episode number with only public episodes taken into account.
    // pub volume_no: i64,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_pages: u32,
}

/// Represents sorting options for episodes.
pub enum Sort {
    /// Sort by ascending order.
    Asc,
    /// Sort by descending order.
    Desc,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        };
        write!(f, "{txt}")
    }
}
