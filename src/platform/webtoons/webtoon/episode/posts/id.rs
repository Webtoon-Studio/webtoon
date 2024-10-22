use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{cmp::Ordering, fmt::Display, num::ParseIntError, str::FromStr};
use thiserror::Error;

use crate::platform::webtoons::meta::ParseLetterError;

use self::base36::Base36;

type Result<T, E = Error> = core::result::Result<T, E>;

/// Represents possible errors when parsing a [`Post`] id.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse `{id}` into `Id`: {context}")]
    InvalidFormat { id: String, context: String },
    #[error("failed to parse `{id}` into `Id`: {error}")]
    InvalidTypeLetter { id: String, error: ParseLetterError },
    #[error("failed to parse `{id}` into `Id`: {error}")]
    ParseNumber { id: String, error: ParseIntError },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct Id {
    // The format follows this pattern: `GW-epicom:0-w_95_1-1d-z`
    // `GW-epicom` can be ignored and seems to just be a namespace. epicom = episode comment.
    // `0` is  an unknown tag
    // `w` is for originals and `c` id for canvas
    // `95` reresents the webtoon id
    // `1` represents the episode
    // `1d` is a base 36, 0-9 + a-z
    // `z` is if for replies. If there is no reply the id will end at `1d`.
    // Post and the reply value cannot be 0.
    // The values also are an indicator of the chronological order in which they are posted.

    // We don't know what this field is in the id but need to keep it
    tag: u32,
    scope: &'static str,
    webtoon: u32,
    episode: u16,
    post: Base36,
    reply: Option<Base36>,
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // split `GW-epicom:0-w_95_1-1d-z` to GW-epicom` and `0-w_95_1-1d-z`
        let id = s
            .split(':')
            // get `0-w_95_1-1d-z`
            .nth(1)
            .ok_or_else(|| Error::InvalidFormat {
                id: s.to_owned(),
                context: "there was no right-hand part after splitting on `:`".to_string(),
            })?;

        // split `0-w_95_1-1d-z` to `0` `w_95_1` `1d` `z`
        let parts: Vec<&str> = id.split('-').collect();

        if parts.len() < 3 {
            return Err(Error::InvalidFormat {
                id: s.to_owned(),
                context: format!(
                    "splitting on `-` should yield at least 3 parts, but only yielded {}",
                    parts.len()
                ),
            });
        }

        let tag: u32 = parts[0].parse().map_err(|err| Error::ParseNumber {
            id: s.to_owned(),
            error: err,
        })?;

        let page_id = parts[1];
        // split `w_95_1` to `w` `95` `1`
        let page_id_parts: Vec<&str> = page_id.split('_').collect();

        if page_id_parts.len() != 3 {
            return Err(Error::InvalidFormat {
                id: s.to_owned(),
                context: format!(
                    r#"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but {page_id} only has {} parts"#,
                    page_id_parts.len()
                ),
            });
        }

        // trick to get a static str from a runtime value
        let scope = match page_id_parts[0] {
            "w" => "w",
            "c" => "c",
            _ => unreachable!("a webtoon can only be either an original or canvas"),
        };

        // parse `95` to u32
        let webtoon = page_id_parts[1].parse().map_err(|err| Error::ParseNumber {
            id: s.to_owned(),
            error: err,
        })?;

        // parse `1` to u16
        let episode = page_id_parts[2].parse().map_err(|err| Error::ParseNumber {
            id: s.to_owned(),
            error: err,
        })?;

        // parse `1d` to `Base36`
        let post = parts[2].parse().map_err(|err| Error::ParseNumber {
            id: s.to_owned(),
            error: err,
        })?;

        // if exists parse `z` to `Base36`
        let reply: Option<Base36> = if parts.len() == 4 {
            Some(parts[3].parse().map_err(|err| Error::ParseNumber {
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
                "GW-epicom:{}-{}_{}_{}-{}-{reply}",
                self.tag, self.scope, self.webtoon, self.episode, self.post,
            )
        } else {
            write!(
                f,
                "GW-epicom:{}-{}_{}_{}-{}",
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
                    (None, Some(other)) => Ordering::Less,

                    // Inverse of the above: If there is a reply for the first one, and the Rhs is None(a direct post)
                    // it must always be greater than the direct post.
                    (Some(reply), None) => Ordering::Greater,

                    // Same direct post
                    (None, None) => Ordering::Equal,
                }
            }
        }
    }
}

impl PartialOrd for Id {
    #[allow(
        clippy::non_canonical_partial_ord_impl,
        reason = "`Id` ordering is only meaningful for the same webtoon on the same episode"
    )]
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

#[cfg(test)]
mod test {
    use pretty_assertions::assert_ne;

    use super::*;

    #[test]
    fn should_be_equal_str() {
        let id = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(49),
            reply: None,
        };

        let id_with_reply = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(49),
            reply: Some(Base36::new(1)),
        };

        // 1d == 49
        pretty_assertions::assert_eq!(id, "GW-epicom:0-w_95_1-1d");
        pretty_assertions::assert_eq!(id_with_reply, "GW-epicom:0-w_95_1-1d-1");
    }

    #[test]
    fn should_be_not_equal_str() {
        let id = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(49),
            reply: None,
        };

        pretty_assertions::assert_ne!(id, "GW-epicom:0-w_95_2-1d");
        pretty_assertions::assert_ne!(id, "GW-epicom:0-w_95_1-1d-1");
    }

    #[test]
    fn should_be_ordered() {
        let forty_nine = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(49),
            reply: None,
        };

        let fifty = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(50),
            reply: None,
        };

        let fifty_with_reply = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(50),
            reply: Some(Base36::new(1)),
        };

        assert!(fifty > forty_nine);
        assert!(forty_nine < fifty);

        // Different webtoons cannot be compared
        assert!(fifty.partial_cmp(&"GW-epicom:0-w_96_1-1d").is_none());
        assert!(fifty.partial_cmp(&"GW-epicom:0-w_96_1-1d-1").is_none());

        // Different episodes cannot be compared
        assert!(fifty.partial_cmp(&"GW-epicom:0-w_95_2-1d").is_none());
        assert!(fifty.partial_cmp(&"GW-epicom:0-w_95_2-1d-1").is_none());

        assert!(fifty > "GW-epicom:0-w_95_1-1d");
        assert!(forty_nine < "GW-epicom:0-w_95_1-1d-1");
        assert!(fifty_with_reply > fifty);
    }

    #[test]
    fn should_turn_post_id_to_string() {
        let id = Id {
            tag: 0,
            scope: "w",
            webtoon: 95,
            episode: 1,
            post: Base36::new(49),
            reply: None,
        };

        pretty_assertions::assert_str_eq!("GW-epicom:0-w_95_1-1d", id.to_string());
    }

    #[test]
    fn should_turn_reply_id_to_string() {
        let id = Id {
            tag: 0,
            scope: "c",
            webtoon: 656_579,
            episode: 161,
            post: Base36::new(35),
            reply: Some(Base36::new(35)),
        };

        pretty_assertions::assert_str_eq!("GW-epicom:0-c_656579_161-z-z", id.to_string());
    }

    #[test]
    fn should_parse_post_id() {
        let id = Id::from_str("GW-epicom:0-w_95_1-1d").unwrap();

        pretty_assertions::assert_eq!(id.scope, "w");
        pretty_assertions::assert_eq!(id.webtoon, 95);
        pretty_assertions::assert_eq!(id.episode, 1);
        pretty_assertions::assert_eq!(id.post, 49);
        pretty_assertions::assert_eq!(id.reply, None);
    }

    #[test]
    fn should_parse_reply_id() {
        {
            let id = Id::from_str("GW-epicom:0-w_95_1-1d-z").unwrap();

            pretty_assertions::assert_eq!(id.scope, "w");
            pretty_assertions::assert_eq!(id.webtoon, 95);
            pretty_assertions::assert_eq!(id.episode, 1);
            pretty_assertions::assert_eq!(id.post, 49);
            pretty_assertions::assert_eq!(id.reply, Some(Base36::new(35)));
        }
        {
            let id = Id::from_str("GW-epicom:0-c_656579_161-13-1").unwrap();

            pretty_assertions::assert_eq!(id.scope, "c");
            pretty_assertions::assert_eq!(id.webtoon, 656_579);
            pretty_assertions::assert_eq!(id.episode, 161);
            pretty_assertions::assert_eq!(id.post, 39);
            pretty_assertions::assert_eq!(id.reply, Some(Base36::new(1)));
        }
    }
}
