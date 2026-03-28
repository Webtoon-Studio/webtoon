#![allow(dead_code)]

use crate::{
    platform::webtoons::{
        error::PostsError,
        webtoon::{
            episode::Episode,
            post::{Body, Flare, Giphy, Post, Poster, Reaction, Sticker, id::Id},
        },
    },
    stdx::{
        cache::Cache,
        error::{Assume, assumption},
    },
};
use chrono::DateTime;
use serde::Deserialize;
use std::{str::FromStr, sync::Arc};

#[derive(Deserialize)]
pub struct RawPostResponse {
    pub result: RawResult,
    // "success"
    pub status: String,
}

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub next: Option<Id>,
    // QUESTION: `prev` might always be Some
    pub prev: Option<Id>,
}

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
    // NOTE: this could be the same as `is_owner`, but it's hard to tell for sure.
    // For now, this is what we use to indicate if a post was left by current session user.
    pub is_owner_account: bool,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reactions {
    pub content_id: Id,
    pub emotions: Vec<Emotions>,
    pub reaction_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emotions {
    pub count: u32,
    pub emotion_id: String,
    pub reacted: bool,
}

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restriction {
    pub is_blind_post_restricted: bool,
    // If blocked
    pub is_write_post_restricted: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionGroup {
    pub sections: Vec<Section>,
    pub total_count: u64,
}

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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GiphyData {
    pub giphy_id: String,
    title: String,
    // rendering: unimplemented!(),
    // thumbnail: unimplemented!(),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StickerData {
    pub sticker_id: String,
    pub sticker_pack_id: String,
    domain: String,
    path: String,
    height: u16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetaData {
    content_type: String,
    content_sub_type: String,
    content_id: String,
    // If webtoon is deleted, then this will be empty.
    pub info: Option<ContentInfo>,
}

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

#[allow(clippy::struct_field_names)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SuperLikeData {
    pub super_like_count: u32,
    pub super_like_price: u32,
    pub super_like_received_at: i64,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountResult {
    pub content_id: Id,
    pub emotions: Vec<Emotions>,
}

impl TryFrom<(&Episode, RawPost)> for Post {
    type Error = PostsError;

    #[allow(clippy::too_many_lines)]
    fn try_from((episode, post): (&Episode, RawPost)) -> Result<Self, Self::Error> {
        let emotions = post
            .reactions
            .first()
            .assumption(
                "`reactions` field in `webtoons.com` raw post response didn't have a 0th element",
            )?
            .emotions
            .as_slice();

        let votes = |id: &str, target: &mut bool| {
            emotions
                .iter()
                .find(|e| e.emotion_id == id)
                .map(|e| {
                    *target = e.reacted;
                    e.count
                })
                .unwrap_or_default()
        };

        let mut liked: bool = false;
        let mut disliked: bool = false;

        let upvotes = votes("likes", &mut liked);
        let downvotes = votes("dislikes", &mut disliked);

        assumption!(
            !(liked && disliked),
            "user cannot both have liked *and* disliked a post on `webtoons.com`; either or, neither, but not both"
        );

        // The way `webtoons.com` keeps track of a like or dislike guarantees(?): they are mutually exclusive.
        let reaction = match (liked, disliked) {
            (true, _) => Reaction::Upvote,
            (_, true) => Reaction::Downvote,
            _ => Reaction::None,
        };

        let Some(posted) = DateTime::from_timestamp_millis(post.created_at) else {
            assumption!(
                "timestamps returned from `webtoons.com` posts api should always be a valid unix millisecond timestamp, got `{}`",
                post.created_at
            );
        };

        let mut webtoons = Vec::new();
        let mut super_like: Option<u32> = None;
        let mut giphy_or_sticker = None;

        for section in post.section_group.sections {
            match section {
                Section::Giphy { data, .. } => {
                    assumption!(
                        giphy_or_sticker.is_none(),
                        "should always be `None`, as only one kind of flare can be added to a post at once. If this is `Some`, then that means there was multiple flares, and now we must handle that"
                    );
                    giphy_or_sticker = Some(Flare::Giphy(Giphy::new(data.giphy_id.clone())));
                }
                Section::Sticker { data, .. } => match Sticker::from_str(&data.sticker_id) {
                    Ok(sticker) => {
                        assumption!(
                            giphy_or_sticker.is_none(),
                            "should always be `None`, as only one kind of flare can be added to a post at once. If this is `Some`, then that means there was multiple flares, and now we must handle that"
                        );
                        giphy_or_sticker = Some(Flare::Sticker(sticker));
                    }
                    Err(err) => assumption!(
                        "`webtoons.com` post sticker id (returned from `webtoons.com`) should always be a valid id: {err}\n\n{}",
                        data.sticker_id
                    ),
                },
                Section::ContentMeta { data, .. } => {
                    let Some(path) = data
                        .info
                        .as_ref()
                        .map(|info| info.extra.episode_list_path.as_str())
                    else {
                        break;
                    };

                    let url = match url::Url::parse("https://www.webtoons.com")
                        .assumption("`https://www.webtoons.com` should be a valid url")?
                        .join(path)
                    {
                        Ok(url) => url,
                        Err(err) => assumption!(
                            "`https://www.webtoons.com` should join with `episode_list_path` (returned by `webtoons.com`) to create a valid url: {err}\n\n{path}"
                        ),
                    };

                    let webtoon = match episode.webtoon.client.webtoon_from_url(url.as_str()) {
                        Ok(webtoon) => webtoon,
                        Err(err) => assumption!(
                            "url formed by joining known good base with returned data from `webtoons.com` should always yield a valid Webtoon homepage url: {err}\n\n{url}"
                        ),
                    };

                    webtoons.push(webtoon);
                }
                Section::SuperLike { data, .. } => {
                    super_like = Some(data.super_like_count);
                }
            }
        }

        let flare = if webtoons.is_empty() {
            giphy_or_sticker
        } else {
            Some(Flare::Webtoons(webtoons))
        };

        Ok(Self {
            episode: episode.clone(),
            id: post.id,
            parent_id: post.root_id,
            body: Body {
                contents: Arc::from(post.body),
                flare,
                is_spoiler: post.settings.spoiler_filter == "ON",
            },
            upvotes,
            downvotes,
            replies: post.child_post_count,
            is_deleted: post.status == "DELETE",
            posted,
            poster: Poster {
                episode: episode.number,
                post_id: post.id,
                cuid: Arc::from(post.created_by.cuid),
                profile: Arc::from(post.created_by.profile_url),
                username: Arc::from(post.created_by.name),
                is_current_session_user: post.is_owner_account,
                is_current_webtoon_creator: post.created_by.is_page_owner,
                is_creator: post.created_by.is_creator,
                is_blocked: post.created_by.restriction.is_write_post_restricted,
                reaction: Cache::new(reaction),
                super_like,
            },
        })
    }
}
