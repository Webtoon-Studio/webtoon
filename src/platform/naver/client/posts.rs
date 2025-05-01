use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Posts {
    pub result: Result,
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub best_list: Vec<CommentList>,
    pub comment_list: Vec<CommentList>,
    pub count: Count,
    pub page_model: PageModel,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Count {
    pub comment: u32,
    pub reply: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageModel {
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentList {
    pub comment_no: String,
    pub parent_comment_no: String,
    pub user_name: String,
    pub contents: String,
    pub sympathy_count: u32,
    pub antipathy_count: u32,
    pub reply_count: u32,
    pub best: bool,
    pub mod_time: String,

    // NOTE: Some of these can be null sometimes.
    pub id_no: Option<String>,
    pub user_id_no: Option<String>,
    pub profile_user_id: Option<String>,

    pub manager: bool,
    pub deleted: bool,
    pub expose: bool,
    pub anonymous: bool,
    pub visible: bool,
    pub mine: bool,
}
