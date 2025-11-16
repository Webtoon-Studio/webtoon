use anyhow::{Context, bail};
use chrono::DateTime;
use serde::Deserialize;
use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;

use crate::platform::webtoons::webtoon::post::id::Id;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct RawPostResponse {
    pub result: RawResult,
    // "success"
    pub status: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawResult {
    #[serde(default)]
    pub active_post_count: u32,
    #[serde(default)]
    pub active_root_post_count: u32,
    pub is_page_owner: bool,
    pub pagination: Pagination,
    #[serde(default)]
    pub post_count: u32,
    pub posts: Vec<RawPost>,
    #[serde(default)]
    pub root_post_count: u32,
    #[serde(default)]
    pub tops: Vec<RawPost>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub next: Option<Id>,
    // prev might always be Some
    pub prev: Option<Id>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPost {
    pub active_child_post_count: i32,
    pub active_page_owner_child_post_count: i32,
    pub body: String,
    pub body_format: BodyFormat,
    pub child_post_count: u32,
    pub comment_depth: u8,
    pub created_at: i64,
    pub created_by: CreatedBy,
    // "BY_USER"
    pub creation_type: String,
    pub depth: u8,
    // pub extraList: Vec<_>,
    pub id: Id,
    pub is_owner: bool,
    pub is_pinned: bool,
    pub page_id: String,
    pub page_owner_child_post_count: i32,
    pub page_url: String,
    pub reactions: Vec<Reactions>,
    pub root_id: Id,
    pub section_group: SectionGroup,
    // "epicom"
    pub service_ticket_id: String,
    pub settings: Settings,
    // "SERVICE" "DELETE" "END"
    pub status: String,
    pub updated_at: i64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedBy {
    pub cuid: String,
    pub enc_user_id: String,
    // pub // extraList: Vec<_>,
    pub id: String,
    pub is_creator: bool,
    pub is_page_owner: bool,
    pub masked_user_id: String,
    pub name: String,
    pub profile_image: ProfileImage,
    pub profile_url: String,
    // "PAGE"
    pub publisher_type: String,
    pub restriction: Restriction,
    pub status: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reactions {
    pub content_id: Id,
    pub emotions: Vec<Emotions>,
    pub reaction_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emotions {
    pub count: u32,
    pub emotion_id: String,
    pub reacted: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyFormat {
    // "PLAIN"
    pub r#type: String,
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileImage {}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restriction {
    pub is_blind_post_restricted: bool,
    // If blocked
    pub is_write_post_restricted: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionGroup {
    pub sections: Vec<Section>,
    pub total_count: u64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    // "ON" "OFF"
    pub reaction: String,
    // "ON" "OFF"
    pub reply: String,
    // "ON" "OFF"
    pub spoiler_filter: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "sectionType")]
#[serde(rename_all_fields = "camelCase")]
pub enum Section {
    #[serde(rename = "GIPHY")]
    Giphy { section_id: String, data: GiphyData },
    #[serde(rename = "STICKER")]
    Sticker {
        section_id: String,
        data: StickerData,
    },
    #[serde(rename = "CONTENT_META")]
    ContentMeta {
        section_id: String,
        data: ContentMetaData,
    },
    #[serde(rename = "SUPER_LIKE")]
    SuperLike {
        section_id: String,
        data: SuperLikeData,
    },
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GiphyData {
    pub giphy_id: String,
    title: String,
    // rendering: unimplemented!(),
    // thumbnail: unimplemented!(),
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StickerData {
    pub sticker_id: String,
    pub sticker_pack_id: String,
    domain: String,
    path: String,
    height: u16,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetaData {
    content_type: String,
    content_sub_type: String,
    content_id: String,
    pub info: ContentInfo,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct ContentInfo {
    name: String,
    pub extra: Extra,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub episode_list_path: String,
}

#[allow(unused, clippy::struct_field_names)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SuperLikeData {
    pub super_like_count: u32,
    pub super_like_price: u32,
    pub super_like_received_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Count {
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

impl
    TryFrom<(
        &crate::platform::webtoons::webtoon::episode::Episode,
        RawPost,
    )> for crate::platform::webtoons::webtoon::post::Post
{
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(
        (episode, post): (
            &crate::platform::webtoons::webtoon::episode::Episode,
            RawPost,
        ),
    ) -> Result<Self, Self::Error> {
        let mut did_like: bool = false;
        let mut did_dislike: bool = false;

        let upvotes = post
            .reactions
            .first()
            .context("`reactions` field didn't have a 0th element and it should always have one")?
            .emotions
            .iter()
            .find(|emotion| emotion.emotion_id == "like")
            .map(|likes| {
                did_like = likes.reacted;
                likes.count
            })
            .unwrap_or_default();

        let downvotes = post
            .reactions
            .first()
            .context("`reactions` field didn't have a 0th element and it should always have one")?
            .emotions
            .iter()
            .find(|emotion| emotion.emotion_id == "dislike")
            .map(|dislikes| {
                did_dislike = dislikes.reacted;
                dislikes.count
            })
            .unwrap_or_default();

        // The way webtoons keeps track of a like or dislike guarantees(?) they are mutually exclusive.
        let reaction = if did_like {
            crate::platform::webtoons::webtoon::post::Reaction::Upvote
        } else if did_dislike {
            crate::platform::webtoons::webtoon::post::Reaction::Downvote
        } else {
            // Defaults to `None` if no session was available for use.
            crate::platform::webtoons::webtoon::post::Reaction::None
        };

        let is_deleted = post.status == "DELETE";
        let is_spoiler = post.settings.spoiler_filter == "ON";
        let mut super_like: Option<u32> = None;

        // Only Webtoon flare can have multiple.
        // Super likes might be able to exist along with other flare?
        let flare = if post.section_group.sections.len() > 1 {
            let mut webtoons = Vec::new();
            for section in post.section_group.sections {
                match section {
                    Section::ContentMeta { data, .. } => {
                        let url = format!(
                            "https://www.webtoons.com{}",
                            data.info.extra.episode_list_path
                        );
                        let webtoon = episode.webtoon.client.webtoon_from_url(&url)?;
                        webtoons.push(webtoon);
                    }
                    Section::SuperLike { data, .. } => {
                        super_like = Some(data.super_like_count);
                    }
                    _ => {
                        bail!(
                            "Only the Webtoon meta flare can have more than one in the section group, yet encountered a case where there was more than one of another flare type: {section:?}"
                        );
                    }
                }
            }
            Some(crate::platform::webtoons::webtoon::post::Flare::Webtoons(
                webtoons,
            ))
        } else {
            match post.section_group.sections.first() {
                Some(section) => match section {
                    Section::Giphy { data, .. } => {
                        Some(crate::platform::webtoons::webtoon::post::Flare::Giphy(
                            crate::platform::webtoons::webtoon::post::Giphy::new(
                                data.giphy_id.clone(),
                            ),
                        ))
                    }
                    Section::Sticker { data, .. } => {
                        let sticker = crate::platform::webtoons::webtoon::post::Sticker::from_str(
                            &data.sticker_id,
                        )
                        .context("Failed to parse sticker id")?;
                        Some(crate::platform::webtoons::webtoon::post::Flare::Sticker(
                            sticker,
                        ))
                    }
                    Section::ContentMeta { data, .. } => {
                        let url = format!(
                            "https://www.webtoons.com{}",
                            data.info.extra.episode_list_path
                        );
                        let webtoon = episode.webtoon.client.webtoon_from_url(&url)?;
                        Some(crate::platform::webtoons::webtoon::post::Flare::Webtoons(
                            vec![webtoon],
                        ))
                    }
                    // Ignore super likes
                    Section::SuperLike { data, .. } => {
                        super_like = Some(data.super_like_count);
                        None
                    }
                },
                None => None,
            }
        };

        Ok(Self {
            episode: episode.clone(),
            id: post.id,
            parent_id: post.root_id,
            body: crate::platform::webtoons::webtoon::post::Body {
                contents: Arc::from(post.body),
                flare,
                is_spoiler,
            },
            upvotes,
            downvotes,
            replies: post.child_post_count,
            is_top: post.is_pinned,
            is_deleted,
            posted: DateTime::from_timestamp_millis(post.created_at).with_context(|| {
                format!(
                    "`{}` is not a valid unix millisecond timestamp",
                    post.created_at
                )
            })?,
            poster: crate::platform::webtoons::webtoon::post::Poster {
                webtoon: episode.webtoon.clone(),
                episode: episode.number,
                post_id: post.id,
                cuid: Arc::from(post.created_by.cuid),
                profile: Arc::from(post.created_by.profile_url),
                username: Arc::from(post.created_by.name),
                is_current_session_user: post.created_by.is_page_owner,
                is_current_webtoon_creator: post.created_by.is_page_owner,
                is_creator: post.created_by.is_creator,
                is_blocked: post.created_by.restriction.is_write_post_restricted,
                reaction: Arc::new(RwLock::new(reaction)),
                super_like,
            },
        })
    }
}
