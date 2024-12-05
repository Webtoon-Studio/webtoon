use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::platform::naver) struct Response {
    pub result: Result,
    pub success: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct Result {
    #[serde(rename = "bestList")]
    pub best: Vec<CommentList>,
    #[serde(rename = "commentList")]
    pub comments: Vec<CommentList>,
    pub count: Count,
    pub page_model: PageModel,
    pub sort: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct CommentList {
    #[serde(rename = "antipathyCount")]
    pub downvotes: u32,
    pub best: bool,
    pub comment_no: String,
    pub contents: String,
    pub country: String,
    pub deleted: bool,
    pub following: bool,
    pub lang: String,
    pub manager: bool,
    pub mine: bool,
    pub reg_time_gmt: String,
    pub object_id: String,
    pub parent_comment_no: String,
    pub reply_all_count: u32,
    pub reply_count: u32,
    pub sort_value: i64,
    #[serde(rename = "sympathyCount")]
    pub upvotes: u32,
    pub user_blocked: bool,
    pub profile_user_id: Option<String>,
    pub id_no: Option<String>,
    pub user_id_no: Option<String>,
    #[serde(rename = "userName")]
    pub username: String,
}

#[derive(Deserialize)]
pub(in crate::platform::naver) struct Count {
    #[serde(rename = "comment")]
    pub comments: u32,
    #[serde(rename = "reply")]
    pub replies: u32,
    pub total: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct PageModel {
    pub page: i64,
    pub total_pages: i64,
}
