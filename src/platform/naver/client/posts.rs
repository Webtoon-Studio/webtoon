use serde::Deserialize;

pub use id::Id;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct PostsResult {
    pub result: Result,
    // "success"
    pub status: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    #[serde(default)]
    pub active_post_count: u32,
    #[serde(default)]
    pub active_root_post_count: u32,
    pub is_page_owner: bool,
    pub pagination: Option<Pagination>,
    #[serde(default)]
    pub post_count: u32,
    #[serde(default)]
    pub posts: Vec<Post>,
    #[serde(default)]
    pub root_post_count: u32,
    #[serde(default)]
    pub tops: Vec<Post>,
    #[serde(default)]
    pub pins: Vec<Post>,
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
pub struct Post {
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
    // pub // extraList: Vec<_>,
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

pub mod id {
    use serde::{Deserialize, Serialize};
    use std::{cmp::Ordering, fmt::Display, num::ParseIntError, str::FromStr};
    use thiserror::Error;

    use crate::{platform::webtoons::meta::ParseLetterError, stdx::base36::Base36};

    type Result<T, E = ParseIdError> = core::result::Result<T, E>;

    /// Represents possible errors when parsing a posts id.
    #[non_exhaustive]
    #[derive(Error, Debug)]
    pub enum ParseIdError {
        /// Error for an invalid id format.
        #[error("failed to parse `{id}` into `Id`: {context}")]
        InvalidFormat { id: String, context: String },
        #[error("failed to parse `{id}` into `Id`: {error}")]
        InvalidTypeLetter { id: String, error: ParseLetterError },
        #[error("failed to parse `{id}` into `Id`: {error}")]
        ParseNumber { id: String, error: ParseIntError },
    }

    /// Represents a unique identifier for a post or comment on a Webtoon episode.
    ///
    /// The `Id` struct follows a specific format to uniquely identify a post or a reply in a Webtoon episode's comment
    /// section. The format contains multiple components, each representing a different aspect of the Webtoon, episode,
    /// post, and any potential reply. It also provides information about the chronological order of the comments.
    ///
    /// ### Structure:
    ///
    /// The format of the ID follows this pattern:
    /// `KW-comic_webtoon:0-webtoon_840593_1-31-1`
    ///
    /// - **`KW-comic_webtoon`**:
    ///   This prefix can be ignored and seems to serve as a namespace.
    ///
    /// - **`0`**:
    ///   This is an unknown tag. Its purpose remains unclear, but it is preserved in the ID structure for compatibility.
    ///
    /// - **`webtoon` / `challenge`**:
    ///   This denotes whether the Webtoon is **Featured** (`webtoon`) or **Challenge** (`challenge`) Webtoon.
    ///
    /// - **`840593`**:
    ///   Represents the Webtoon ID. This value is unique to the Webtoon series.
    ///
    /// - **`1`**:
    ///   Represents the episode number within the Webtoon series.
    ///
    /// - **`31`**:
    ///   A unique identifier for the specific post. It is encoded in **Base36** (using characters `0-9` and `a-z`).
    ///   This value indicates the chronological order of the post within the episode's comments section. Posts and replies cannot have a value of `0`.
    ///
    /// - **`1`**:
    ///   Represents a reply to a post. If this component is missing, the ID refers to a top-level post. If present, it indicates the reply to a specific post, also encoded in **Base36**.
    ///
    /// ### Fields:
    ///
    /// - `tag`:
    ///   An unknown field that is part of the ID structure but its exact purpose is not fully understood. It is included for completeness.
    ///
    /// - `scope`:
    ///   A string representing whether the Webtoon is an **Original** or **Canvas** series (`webtoon` or `challenge`).
    ///
    /// - `webtoon`:
    ///   The unique ID for the Webtoon series.
    ///
    /// - `episode`:
    ///   The episode number within the Webtoon series.
    ///
    /// - `post`:
    ///   The **Base36**-encoded identifier for the specific post.
    ///
    /// - `reply`:
    ///   An optional **Base36**-encoded identifier for a reply to the post. If `None`, the ID refers to a top-level comment.
    ///
    /// ### Notes:
    ///
    /// - The ID structure provides an implicit chronological order, meaning that IDs with lower values (in the `post` or `reply` fields)
    ///   were posted earlier than those with higher values.
    /// - The ID must have non-zero values for both the post and reply components, ensuring that each comment and reply is uniquely identifiable.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
    #[serde(try_from = "String")]
    #[serde(into = "String")]
    pub struct Id {
        tag: u32,
        scope: Scope,
        webtoon: u32,
        episode: u16,
        post: Base36,
        reply: Option<Base36>,
    }

    impl FromStr for Id {
        type Err = ParseIdError;

        fn from_str(s: &str) -> Result<Self> {
            // split `KW-comic_webtoon:0-webtoon_840593_1-31-1` to KW-comic_webtoon` and `0-webtoon_840593_1-31-1`
            let id = s
                .split(':')
                // get `0-webtoon_840593_1-31-1`
                .next_back()
                .ok_or_else(|| ParseIdError::InvalidFormat {
                    id: s.to_owned(),
                    context: "there was no right-hand part after splitting on `:`".to_string(),
                })?;

            // split `0-webtoon_840593_1-31-1` to `0` `webtoon_840593_1` `31` `1`
            let parts: Vec<&str> = id.split('-').collect();

            if parts.len() < 3 {
                return Err(ParseIdError::InvalidFormat {
                    id: s.to_owned(),
                    context: format!(
                        "splitting on `-` should yield at least 3 parts, but only yielded {}",
                        parts.len()
                    ),
                });
            }

            let tag: u32 = parts[0].parse().map_err(|err| ParseIdError::ParseNumber {
                id: s.to_owned(),
                error: err,
            })?;

            let page_id = parts[1];
            // split `webtoon_840593_1` to `weboon` `840593` `1`
            let page_id_parts: Vec<&str> = page_id.split('_').collect();

            if page_id_parts.len() != 3 {
                return Err(ParseIdError::InvalidFormat {
                    id: s.to_owned(),
                    context: format!(
                        r#"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but {page_id} only has {} parts"#,
                        page_id_parts.len()
                    ),
                });
            }

            let scope = match page_id_parts[0] {
                "webtoon" => Scope::Webtoon,
                "challenge" => Scope::Challenge,
                _ => unreachable!("a webtoon can only be either an original or canvas"),
            };

            // parse `840593` to u32
            let webtoon = page_id_parts[1]
                .parse()
                .map_err(|err| ParseIdError::ParseNumber {
                    id: s.to_owned(),
                    error: err,
                })?;

            // parse `1` to u16
            let episode = page_id_parts[2]
                .parse()
                .map_err(|err| ParseIdError::ParseNumber {
                    id: s.to_owned(),
                    error: err,
                })?;

            // parse `31` to `Base36`
            let post = parts[2].parse().map_err(|err| ParseIdError::ParseNumber {
                id: s.to_owned(),
                error: err,
            })?;

            // if exists parse `1` to `Base36`
            let reply: Option<Base36> = if parts.len() == 4 {
                Some(parts[3].parse().map_err(|err| ParseIdError::ParseNumber {
                    id: s.to_owned(),
                    error: err,
                })?)
            } else {
                None
            };

            let id = Self {
                tag,
                scope,
                webtoon,
                episode,
                post,
                reply,
            };

            Ok(id)
        }
    }

    impl Display for Id {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(reply) = &self.reply {
                write!(
                    f,
                    "KW-comic_webtoon:{}-{}_{}_{}-{}-{reply}",
                    self.tag, self.scope, self.webtoon, self.episode, self.post,
                )
            } else {
                write!(
                    f,
                    "KW-comic_webtoon:{}-{}_{}_{}-{}",
                    self.tag, self.scope, self.webtoon, self.episode, self.post
                )
            }
        }
    }

    impl<'a> PartialEq<&'a str> for Id {
        fn eq(&self, other: &&'a str) -> bool {
            Self::from_str(other).map(|id| *self == id).unwrap_or(false)
        }
    }

    impl PartialEq<String> for Id {
        fn eq(&self, other: &String) -> bool {
            Self::from_str(other).map(|id| *self == id).unwrap_or(false)
        }
    }

    impl Ord for Id {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self.post.cmp(&other.post) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => {
                    match (self.reply, other.reply) {
                        // Both are replies to the same direct post so a direct compare is easy
                        (Some(reply), Some(other)) => reply.cmp(&other),

                        // If there is no reply number for the first one, it must be a direct post, so if there is any
                        // id that has a reply with a matching post number, it must always be Greater and therefore
                        // `self` must be `Less` than the reply.
                        (None, Some(_)) => Ordering::Less,

                        // Inverse of the above: If there is a reply for the first one, and the Rhs is None(a direct post)
                        // it must always be greater than the direct post.
                        (Some(_), None) => Ordering::Greater,

                        // Same direct post
                        (None, None) => Ordering::Equal,
                    }
                }
            }
        }
    }

    #[allow(
        clippy::non_canonical_partial_ord_impl,
        reason = "`Id` ordering is only meaningful for the same webtoon on the same episode"
    )]
    impl PartialOrd for Id {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            // If not a post on the same webtoons' episode then return `None`.
            // Cannot add `self.tag != other.tag` as its still unknown how this number increments, but given that the other
            // checks are enough to know if the post is on the same weboon and the same episode it should be fine.
            if self.scope != other.scope
                || self.webtoon != other.webtoon
                || self.episode != other.episode
            {
                return None;
            }

            Some(self.cmp(other))
        }
    }

    impl<'a> PartialOrd<&'a str> for Id {
        fn partial_cmp(&self, other: &&'a str) -> Option<std::cmp::Ordering> {
            let Ok(other) = Self::from_str(other) else {
                return None;
            };

            self.partial_cmp(&other)
        }
    }

    impl From<Id> for String {
        fn from(val: Id) -> Self {
            val.to_string()
        }
    }

    impl TryFrom<String> for Id {
        type Error = ParseIdError;

        fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
            Self::from_str(&value)
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Scope {
        Webtoon,
        Challenge,
    }

    impl Display for Scope {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let word = match self {
                Self::Webtoon => "webtoon",
                Self::Challenge => "challenge",
            };

            write!(f, "{word}")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn should_be_equal_str() {
            let id = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(109),
                reply: None,
            };

            pretty_assertions::assert_eq!(id, "KW-comic_webtoon:0-webtoon_840593_1-31");

            let id = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(109),
                reply: Some(Base36::new(1)),
            };

            pretty_assertions::assert_eq!(id, "KW-comic_webtoon:0-webtoon_840593_1-31-1");
        }

        #[test]
        fn should_be_not_equal_str() {
            let id = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(31),
                reply: None,
            };

            pretty_assertions::assert_ne!(id, "KW-comic_webtoon:0-webtoon_840593_1-31");
            pretty_assertions::assert_ne!(id, "KW-comic_webtoon:0-webtoon_840593_1-31-1");
        }

        #[test]
        fn should_be_ordered() {
            let forty_nine = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(49),
                reply: None,
            };

            let fifty = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(50),
                reply: None,
            };

            let fifty_with_reply = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(50),
                reply: Some(Base36::new(1)),
            };

            assert!(fifty > forty_nine);
            assert!(forty_nine < fifty);

            // Different webtoons cannot be compared
            assert!(
                fifty
                    .partial_cmp(&"KW-comic_webtoon:0-webtoon_840583_1-31-1")
                    .is_none()
            );
            assert!(
                fifty
                    .partial_cmp(&"KW-comic_webtoon:0-webtoon_841593_1-31")
                    .is_none()
            );

            // Different episodes cannot be compared
            assert!(
                fifty
                    .partial_cmp(&"KW-comic_webtoon:0-webtoon_840593_10-31-1")
                    .is_none()
            );
            assert!(
                fifty
                    .partial_cmp(&"KW-comic_webtoon:0-webtoon_840593_5-31-1")
                    .is_none()
            );

            assert!(fifty > "KW-comic_webtoon:0-webtoon_840593_1-1d");
            assert!(forty_nine < "KW-comic_webtoon:0-webtoon_840593_1-1e");
            assert!(fifty_with_reply > fifty);
        }

        #[test]
        fn should_turn_post_id_to_string() {
            let id = Id {
                tag: 0,
                scope: Scope::Webtoon,
                webtoon: 840593,
                episode: 1,
                post: Base36::new(109),
                reply: None,
            };

            pretty_assertions::assert_str_eq!(
                "KW-comic_webtoon:0-webtoon_840593_1-31",
                id.to_string()
            );
        }

        #[test]
        fn should_turn_reply_id_to_string() {
            let id = Id {
                tag: 0,
                scope: Scope::Challenge,
                webtoon: 840593,
                episode: 161,
                post: Base36::new(35),
                reply: Some(Base36::new(35)),
            };

            pretty_assertions::assert_str_eq!(
                "KW-comic_webtoon:0-challenge_840593_161-z-z",
                id.to_string()
            );
        }

        #[test]
        fn should_parse_post_id() {
            let id = Id::from_str("KW-comic_webtoon:0-webtoon_840593_1-z").unwrap();

            pretty_assertions::assert_eq!(id.scope, Scope::Webtoon);
            pretty_assertions::assert_eq!(id.webtoon, 840593);
            pretty_assertions::assert_eq!(id.episode, 1);
            pretty_assertions::assert_eq!(id.post, 35);
            pretty_assertions::assert_eq!(id.reply, None);
        }

        #[test]
        fn should_parse_reply_id() {
            {
                let id = Id::from_str("KW-comic_webtoon:0-challenge_840593_1-z-z").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::Challenge);
                pretty_assertions::assert_eq!(id.webtoon, 840593);
                pretty_assertions::assert_eq!(id.episode, 1);
                pretty_assertions::assert_eq!(id.post, 35);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(35)));
            }
            {
                let id = Id::from_str("KW-comic_webtoon:0-challenge_840593_1-z-z").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::Challenge);
                pretty_assertions::assert_eq!(id.webtoon, 840593);
                pretty_assertions::assert_eq!(id.episode, 1);
                pretty_assertions::assert_eq!(id.post, 35);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(35)));
            }
        }
    }
}
