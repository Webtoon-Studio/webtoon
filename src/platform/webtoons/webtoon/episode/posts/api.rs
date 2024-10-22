use anyhow::{Context, Error};
use serde::Deserialize;

use super::id::Id;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Api {
    pub(super) result: Result,
    // "success"
    pub(super) status: String,
}

impl Api {
    pub(super) fn deserialize(response: &str) -> core::result::Result<Self, Error> {
        Ok(serde_json::from_str::<Self>(response)
            .context("Post's api response JSON failed to deserialize")?)
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Result {
    #[serde(default)]
    pub(super) active_post_count: u32,
    #[serde(default)]
    pub(super) active_root_post_count: u32,
    pub(super) is_page_owner: bool,
    pub(super) pagination: Pagination,
    #[serde(default)]
    pub(super) post_count: u32,
    pub(super) posts: Vec<Post>,
    #[serde(default)]
    pub(super) root_post_count: u32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Pagination {
    pub(super) next: Option<Id>,
    // prev might always be Some
    pub(super) prev: Option<Id>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Post {
    pub(super) active_child_post_count: i32,
    pub(super) active_page_owner_child_post_count: i32,
    pub(super) body: String,
    pub(super) body_format: BodyFormat,
    pub(super) child_post_count: u32,
    pub(super) comment_depth: u8,
    pub(super) created_at: i64,
    pub(super) created_by: CreatedBy,
    // "BY_USER"
    pub(super) creation_type: String,
    pub(super) depth: u8,
    // pub(super) // extraList: Vec<_>,
    pub(super) id: Id,
    pub(super) is_owner: bool,
    pub(super) is_pinned: bool,
    pub(super) page_id: String,
    pub(super) page_owner_child_post_count: i32,
    pub(super) page_url: String,
    pub(super) reactions: Vec<Reactions>,
    pub(super) root_id: Id,
    pub(super) section_group: SectionGroup,
    // "epicom"
    pub(super) service_ticket_id: String,
    pub(super) settings: Settings,
    // "SERVICE" "DELETE" "END"
    pub(super) status: String,
    pub(super) updated_at: i64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CreatedBy {
    pub(super) cuid: String,
    pub(super) enc_user_id: String,
    // pub(super) // extraList: Vec<_>,
    pub(super) id: String,
    pub(super) is_creator: bool,
    pub(super) is_page_owner: bool,
    pub(super) masked_user_id: String,
    pub(super) name: String,
    pub(super) profile_image: ProfileImage,
    pub(super) profile_url: String,
    // "PAGE"
    pub(super) publisher_type: String,
    pub(super) restriction: Restriction,
    pub(super) status: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Reactions {
    pub(super) content_id: Id,
    pub(super) emotions: Vec<Emotions>,
    pub(super) reaction_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Emotions {
    pub(super) count: u32,
    pub(super) emotion_id: String,
    pub(super) reacted: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BodyFormat {
    // "PLAIN"
    pub(super) r#type: String,
    pub(super) version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProfileImage {}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Restriction {
    pub(super) is_blind_post_restricted: bool,
    // If blocked
    pub(super) is_write_post_restricted: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SectionGroup {
    // sections: Vec<Section>,
    pub(super) total_count: u64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Settings {
    // "ON" "OFF"
    pub(super) reaction: String,
    // "ON" "OFF"
    pub(super) reply: String,
    // "ON" "OFF"
    pub(super) spoiler_filter: String,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct Section {
//     data: Data,
//     priority: u8,
//     section_id: String,
//     section_type: SectionType,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// enum SectionType {
//     Giphy,
//     ContentMeta,
//     Sticker,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct Data {
//     giphy_id: String,
//     title: String,
//     rendering: String,
// }

// Get new count: GET
// {
//     "status": "success",
//     "result": {
//         "contentId": "GW-epicom:0-c_843910_4-3",
//         "emotions": [
//             {
//                 "emotionId": "like",
//                 "count": 1,
//                 "reacted": true
//             }
//         ]
//     }
// }
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Count {
    pub status: String,
    pub result: CountResult,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountResult {
    pub content_id: Id,
    pub emotions: Vec<Emotions>,
}
