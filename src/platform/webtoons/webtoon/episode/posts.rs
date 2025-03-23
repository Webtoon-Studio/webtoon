//! Module containing things related to posts and their posters.

use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, Utc};
use core::fmt;
use serde_json::json;
use std::{cmp::Ordering, collections::HashSet, hash::Hash, str::FromStr, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

// Id will now be in `episode::posts` documentation
pub use crate::platform::webtoons::client::posts::Id;

//Stickers for all stickers https://www.webtoons.com/p/api/community/v1/sticker/pack/wt_001 Needs Service-Ticket-Id: epicom

// GIF search
// https://www.webtoons.com/p/api/community/v1/gifs/search?q=happy&offset=0&limit=10

// POST GIF comment
// {
//   "pageId": "c_843910_1",
//   "settings": {
//     "reply": "ON",
//     "reaction": "ON",
//     "spoilerFilter": "OFF"
//   },
//   "title": "",
//   "body": "GIF Comment",
//   "sectionGroup": {
//     "sections": [
//       {
//         "sectionType": "GIPHY",
//         "data": {
//           "giphyId": "fUQ4rhUZJYiQsas6WD"
//         }
//       }
//     ]
//   }
// }

// POST for comment with sticker
// {
//   "pageId": "c_843910_1",
//   "settings": {
//     "reply": "ON",
//     "reaction": "ON",
//     "spoilerFilter": "OFF"
//   },
//   "title": "",
//   "body": "Sticker Comment 2",
//   "sectionGroup": {
//     "sections": [
//       {
//         "sectionType": "STICKER",
//         "data": {
//           "stickerPackId": "wt_001",
//           "stickerId": "wt_001-v2-1"
//         }
//       }
//     ]
//   }
// }

// POST webtoon comment
// {
//   "pageId": "c_843910_1",
//   "settings": {
//     "reply": "ON",
//     "reaction": "ON",
//     "spoilerFilter": "OFF"
//   },
//   "title": "",
//   "body": "Webtoon Comment",
//   "sectionGroup": {
//     "sections": [
//       {
//         "sectionType": "CONTENT_META",
//         "data": {
//           "contentType": "TITLE",
//           "contentSubType": "WEBTOON",
//           "contentId": "95"
//         }
//       }
//     ]
//   }
// }
// Multiple webtoons(only one that can be multiple)
// {
//   "pageId": "c_843910_1",
//   "settings": {
//     "reply": "ON",
//     "reaction": "ON",
//     "spoilerFilter": "OFF"
//   },
//   "title": "",
//   "body": "Multiple Webtoon Comment",
//   "sectionGroup": {
//     "sections": [
//       {
//         "sectionType": "CONTENT_META",
//         "data": {
//           "contentType": "TITLE",
//           "contentSubType": "WEBTOON",
//           "contentId": "5557"
//         }
//       },
//       {
//         "sectionType": "CONTENT_META",
//         "data": {
//           "contentType": "TITLE",
//           "contentSubType": "WEBTOON",
//           "contentId": "95"
//         }
//       }
//     ]
//   }
// }
// contentSubType can also be "CHALLENGE"

use crate::{
    platform::webtoons::{
        self, Webtoon,
        client::posts::{Count, PostsResult, Section},
        errors::{ClientError, PostError, PosterError, ReplyError},
        meta::Scope,
    },
    private::Sealed,
};

use super::Episode;

/// Represents a collection of posts.
///
/// A wrapper for a `Vec<Post>` to provide methods on to further interact.
#[derive(Debug, Clone)]
pub struct Posts {
    pub(super) posts: Vec<Post>,
}

impl Posts {
    /// Creates an iterator which uses a closure to determine if an element
    /// should be yielded.
    ///
    /// Given an element the closure must return `true` or `false`. The returned
    /// iterator will yield only the elements for which the closure returns
    /// true.
    pub fn filter<P>(self, predicate: P) -> impl Iterator<Item = Post>
    where
        P: FnMut(&Post) -> bool,
    {
        self.into_iter().filter(predicate)
    }

    /// Sorts the posts with a comparison function, **without** preserving the initial order of
    /// equal elements.
    ///
    /// This sort is unstable (i.e., may reorder equal elements), in-place (i.e., does not
    /// allocate), and *O*(*n* \* log(*n*)) worst-case.
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Post, &Post) -> Ordering,
    {
        self.posts.sort_unstable_by(compare);
    }

    /// Performs an inplace, unstable sort of the post episode number in an descending order.
    pub fn sort_by_episode_desc(&mut self) {
        self.posts
            .sort_unstable_by(|a, b| b.episode.number.cmp(&a.episode.number));
    }

    /// Performs an inplace, unstable sort of the post episode number in an ascending order.
    pub fn sort_by_episode_asc(&mut self) {
        self.posts
            .sort_unstable_by(|a, b| a.episode.number.cmp(&b.episode.number));
    }

    /// Performs an inplace, unstable sort of the post date, from newest to oldest.
    pub fn sort_by_newest(&mut self) {
        self.posts.sort_unstable_by(|a, b| b.posted.cmp(&a.posted));
    }

    /// Performs an inplace, unstable sort of the post date, from oldest to newest.
    pub fn sort_by_oldest(&mut self) {
        self.posts.sort_unstable_by(|a, b| a.posted.cmp(&b.posted));
    }

    /// Performs an inplace, unstable sort of the upvotes , from largest to smallest.
    pub fn sort_by_upvotes(&mut self) {
        self.posts
            .sort_unstable_by(|a, b| b.upvotes.cmp(&a.upvotes));
    }

    /// Return the underlying `Vec<Post>` as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[Post] {
        &self.posts
    }
}

// Replies for post
//GET https://www.webtoons.com/p/api/community/v2/post/GW-epicom:0-c_843910_1-k/child-posts?sort=oldest&displayBlindCommentAsService=false&prevSize=0&nextSize=10&withCursor=false&offsetPostId=

/// Represensts a post on `webtoons.com`, either a reply or a top-level comment.
#[derive(Clone)]
pub struct Post {
    pub(crate) episode: Episode,
    pub(crate) id: Id,
    pub(crate) parent_id: Id,
    pub(crate) body: Body,
    pub(crate) upvotes: u32,
    pub(crate) downvotes: u32,
    pub(crate) replies: u32,
    pub(crate) is_top: bool,
    pub(crate) is_deleted: bool,
    pub(crate) posted: DateTime<Utc>,
    pub(crate) poster: Poster,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Post")
            // omitting `episode`
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("body", &self.body)
            .field("upvotes", &self.upvotes)
            .field("downvotes", &self.downvotes)
            .field("replies", &self.replies)
            .field("is_top", &self.is_top)
            .field("is_deleted", &self.is_deleted)
            .field("posted", &self.posted)
            .field("poster", &self.poster)
            .finish()
    }
}

impl Post {
    /// Returns the [`Poster`] of post.
    #[must_use]
    pub fn poster(&self) -> &Poster {
        &self.poster
    }

    /// Returns the unique [`Id`] for the post.
    ///
    /// The returned [`Id`] contains all the necessary information to uniquely identify the post
    /// in the context of a specific Webtoon episode. This includes the Webtoon ID,
    /// episode number, post identifier, and optionally a reply identifier if the post is a reply.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// let posts = webtoon.posts().await?;
    /// for post in  posts {
    ///     println!("Post ID: {:?}", post.id());
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The [`Id`] is a composite structure that reflects the internal format used by Webtoon's system.
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns the parent [`Id`] of the post.
    ///
    /// If the post is a top-level comment, the parent ID will be the same as the post's own [`Self::id`].
    /// If the post is a reply to another comment, the parent ID will reflect the ID of the post it is replying to.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// let parent_id = post.parent_id();
    /// if parent_id == post.id() {
    ///     println!("This is a top-level comment.");
    /// } else {
    ///     println!("This is a reply to post with ID: {:?}", parent_id);
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This method is useful for determining whether a post is a top-level comment or a reply to another comment.
    #[must_use]
    pub fn parent_id(&self) -> Id {
        self.parent_id
    }

    /// Returns a reference to the [`Body`] of the post.
    ///
    /// This method provides access to the content of the post and whether it contains spoilers.
    /// The body contains the actual text of the post along with a flag indicating if it is marked as a spoiler.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// let body = post.body();
    /// println!("Post content: {}", body.contents());
    /// println!("Contains spoilers: {}", body.is_spoiler());
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Returns how many upvotes the post has.
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    /// Returns how many downvotes the post has.
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        self.downvotes
    }

    // /// Returns the amount of replies on the post.
    // #[must_use]
    // pub fn reply_count(&self) -> u32 {
    //     self.replies
    // }

    /// Returns whether this post is a top-level comment and not a reply.
    #[must_use]
    pub fn is_comment(&self) -> bool {
        self.id == self.parent_id
    }

    /// Returns whether this post is a reply and not a top-level comment.
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.id != self.parent_id
    }

    /// Returns whether this post is a `TOP` post, one of the pinned top three posts on the episode.
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.is_top
    }

    /// Returns whether this post was deleted.
    ///
    /// One thing to keep in mind is that if a top-level post was deleted and no replies were left,
    /// or if all replies were themselves deleted, it wont be returned in the `Episode::posts` response.
    ///
    /// This will only return `true` if there is a top-level post that has replies on it. Otherwise will return `false`.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Returns the episode number of the post was left on.
    #[must_use]
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in an ISO 8601 millisecond timestamp format.
    #[must_use]
    pub fn posted(&self) -> i64 {
        self.posted.timestamp_millis()
    }

    /// Upvotes post via users session.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    pub async fn upvote(&self) -> Result<(u32, u32), PostError> {
        if self.poster.is_current_session_user {
            // If is_owner is true then user is trying to upvote their own post which is not allowed
            return self.upvotes_and_downvotes().await;
        }

        let reaction = self.poster.reaction.read().await;
        match *reaction {
            Reaction::Upvote => {
                return self.upvotes_and_downvotes().await;
            }
            Reaction::Downvote => {
                // Must first remove downvote before we can upvote
                // Drop read lock
                drop(reaction);
                self.unvote().await?;
            }
            Reaction::None => {
                // Drop read lock
                drop(reaction);
            }
        }

        self.episode
            .webtoon
            .client
            .put_react_to_post(self, Reaction::Upvote)
            .await?;

        let mut reaction = self.poster.reaction.write().await;
        *reaction = Reaction::None;
        drop(reaction);

        self.upvotes_and_downvotes().await
    }

    /// Downvotes post via users session.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    pub async fn downvote(&self) -> Result<(u32, u32), PostError> {
        if self.poster.is_current_session_user {
            // If is_owner is true then user is trying to downvote their own post which is not allowed
            return self.upvotes_and_downvotes().await;
        }

        let reaction = self.poster.reaction.read().await;
        match *reaction {
            // Must first remove upvote before we can upvote
            Reaction::Upvote => {
                // Drop read lock
                drop(reaction);
                self.unvote().await?;
            }
            Reaction::Downvote => {
                return self.upvotes_and_downvotes().await;
            }
            Reaction::None => {
                // Drop read lock
                drop(reaction);
            }
        }

        self.episode
            .webtoon
            .client
            .put_react_to_post(self, Reaction::Downvote)
            .await?;

        let mut reaction = self.poster.reaction.write().await;
        *reaction = Reaction::None;
        drop(reaction);

        self.upvotes_and_downvotes().await
    }

    /// Will clear any upvote or downvote the user might have on the post.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    pub async fn unvote(&self) -> Result<(u32, u32), PostError> {
        let page_id = format!(
            "{}_{}_{}",
            match self.episode.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            self.episode.webtoon.id,
            self.episode.number
        );

        let reaction = self.poster.reaction.read().await;
        let url = match *reaction {
            Reaction::Upvote => format!(
                "https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/like",
                self.id
            ),
            Reaction::Downvote => format!(
                "https://www.webtoons.com/p/api/community/v2/reaction/post_like/channel/{page_id}/content/{}/emotion/dislike",
                self.id
            ),
            Reaction::None => return self.upvotes_and_downvotes().await,
        };
        // Drop read lock
        drop(reaction);

        let token = self.episode.webtoon.client.get_api_token().await?;

        let session = self
            .episode
            .webtoon
            .client
            .session
            .as_ref()
            .map(|session| session.to_string())
            .ok_or(ClientError::NoSessionProvided)?;

        self.episode
            .webtoon
            .client
            .http
            .delete(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Referer", "https://www.webtoons.com/")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Api-Token", token)
            .send()
            .await?;

        let mut reaction = self.poster.reaction.write().await;
        *reaction = Reaction::None;
        drop(reaction);

        self.upvotes_and_downvotes().await
    }

    /// Returns the upvote and downvote count for the post.
    ///
    /// # Returns
    ///
    /// A tuple of `(upvotes, downvotes)`
    ///
    /// # Errors
    ///
    /// Will return an error if there is an issue with the request or deserialzation of the request.
    pub async fn upvotes_and_downvotes(&self) -> Result<(u32, u32), PostError> {
        let response = self
            .episode
            .webtoon
            .client
            .get_upvotes_and_downvotes_for_post(self)
            .await?;

        let text = response.text().await?;

        let count = serde_json::from_str::<Count>(&text).context(text)?;

        if count.status != "success" {
            return Err(PostError::Unexpected(anyhow!("{count:?}")));
        }

        let mut upvotes = 0;
        let mut downvotes = 0;
        for emotion in count.result.emotions {
            if emotion.emotion_id == "like" {
                upvotes = emotion.count;
            } else {
                downvotes = emotion.count;
            }
        }

        Ok((upvotes, downvotes))
    }

    /// Returns the replies on the current post.
    ///
    /// The return type depends on the specified output type and can either return the total number of replies or a collection of the actual replies.
    ///
    /// # Return Types
    ///
    /// - For `u32`: Returns the count of replies.
    /// - For `Posts`: Returns the replies as a [`Posts`] object, with replies sorted from oldest to newest.
    ///
    /// # Usage
    ///
    /// Depending on the type you specify, you can either retrieve the number of replies or the actual replies themselves:
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error, webtoon::episode::posts::Posts};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// let replies: u32 = post.replies().await?;
    /// let replies: Posts = post.replies().await?;
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`PostError`] if there is an issue with the request, such as network issues or deserialization errors.
    pub async fn replies<R: Replies>(&self) -> Result<R, PostError> {
        R::replies(self).await
    }

    /// Posts a reply on top-level comment.
    ///
    /// This method allows users to leave a reply on a top-level comment. The reply can be marked as a spoiler.
    ///
    /// ### Parameters:
    /// - `body`: The content of the comment to be posted.
    /// - `is_spoiler`: A boolean indicating whether the comment should be marked as a spoiler. If `true`, the comment will be marked as a spoiler.
    ///
    /// ### Example:
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::{Error, ReplyError, ClientError}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// match post.reply("In the novel *spoiler*", true).await {
    ///     Ok(_) => println!("left reply!"),
    ///     Err(ReplyError::ClientError(ClientError::InvalidSession | ClientError::NoSessionProvided)) => println!("session issue, failed to leave reply"),
    ///     Err(err) => panic!("{err}"),
    /// }
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors:
    /// - Returns a [`ReplyError`] if there is an issue during the post request.
    pub async fn reply(&self, body: &str, is_spoiler: bool) -> Result<(), ReplyError> {
        if self.is_deleted {
            return Err(ReplyError::DeletedPost);
        };

        self.episode
            .webtoon
            .client
            .post_reply(self, body, is_spoiler)
            .await?;
        Ok(())
    }

    /// Deletes post if the user has permissions to do so.
    ///
    /// If post is already deleted it will short-circuit and return `Ok`.
    ///
    /// ## Permissions
    /// **Own-post**: If the post is from the sessions user, then has permission to delete.
    /// **Wetboon-Owner**: If the current user is the creator of the webtoon the post is on, and thus has moderation capability.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - there is a problem with the request or .  
    /// - there is a serializtion/deserialization error.
    /// - session user has invalid permissions [`PostError::InvalidPermissions`].
    pub async fn delete(&self) -> Result<(), PostError> {
        let user = self
            .episode
            .webtoon
            .client
            .get_user_info_for_webtoon(&self.episode.webtoon)
            .await?;

        // Only perform delete if current post is from current session's user or if they are the creator of the webtoon
        if !(self.poster.is_current_session_user || user.is_webtoon_creator()) {
            return Err(PostError::InvalidPermissions);
        }

        // Return early if already deleted
        if self.is_deleted {
            return Ok(());
        }

        self.episode.webtoon.client.delete_post(self).await?;

        Ok(())
    }
}

impl TryFrom<(&Episode, webtoons::client::posts::Post)> for Post {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(
        (episode, post): (&Episode, webtoons::client::posts::Post),
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
            Reaction::Upvote
        } else if did_dislike {
            Reaction::Downvote
        } else {
            // Defaults to `None` if no session was available for use.
            Reaction::None
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
            Some(Flare::Webtoons(webtoons))
        } else {
            match post.section_group.sections.first() {
                Some(section) => match section {
                    Section::Giphy { data, .. } => {
                        Some(Flare::Giphy(Giphy::new(data.giphy_id.clone())))
                    }
                    Section::Sticker { data, .. } => {
                        let sticker = Sticker::from_str(&data.sticker_id)
                            .context("Failed to parse sticker id")?;
                        Some(Flare::Sticker(sticker))
                    }
                    Section::ContentMeta { data, .. } => {
                        let url = format!(
                            "https://www.webtoons.com{}",
                            data.info.extra.episode_list_path
                        );
                        let webtoon = episode.webtoon.client.webtoon_from_url(&url)?;
                        Some(Flare::Webtoons(vec![webtoon]))
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

        Ok(Post {
            episode: episode.clone(),
            id: post.id,
            parent_id: post.root_id,
            body: Body {
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
            poster: Poster {
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

/// Represents the body of a post, including its content and whether it is marked as a spoiler.
///
/// The body contains the text content of the post, and a flag indicating whether the post contains spoilers.
#[derive(Debug, Clone)]
pub struct Body {
    contents: Arc<str>,
    is_spoiler: bool,
    flare: Option<Flare>,
}

impl Body {
    /// Returns contents of the post body.
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Returns the optional [`Flare`] a post can have.
    ///
    /// This can be a list of Webtoons, a single sticker, or a single giphy gif.
    pub fn flare(&self) -> Option<&Flare> {
        self.flare.as_ref()
    }

    /// Returns whether this post was marked as a spoiler.
    pub fn is_spoiler(&self) -> bool {
        self.is_spoiler
    }
}

/// Represents extra flare that can be added to a post.
///
/// This can be a list of Webtoons, a single sticker, or a single giphy gif.
#[derive(Debug, Clone)]
pub enum Flare {
    /// A GIF in a post.
    Giphy(Giphy),
    /// A list of webtoons in a post.
    Webtoons(Vec<Webtoon>),
    /// A sticker in a post.
    Sticker(Sticker),
}

/// Represents a sticker in a post.
#[derive(Debug, Clone)]
pub struct Sticker {
    pack: String,
    pack_number: u16,
    version: Option<u16>,
    id: u16,
}

impl Sticker {
    /// Returns the sticker's pack id as a String.
    ///
    /// Example: "`wt_001`"
    pub fn pack_id(&self) -> String {
        format!("{}_{:03}", self.pack, self.pack_number)
    }

    /// Returns the sticker's id as a String.
    ///
    /// Example: "`wt_001-v2-1`" (version is optional: "`wt_001-1`")
    pub fn id(&self) -> String {
        match self.version {
            Some(version) => {
                format!(
                    "{}_{:03}-v{version}-{}",
                    self.pack, self.pack_number, self.id
                )
            }
            None => {
                format!("{}_{:03}-{}", self.pack, self.pack_number, self.id)
            }
        }
    }
}

/// Represents an error that can happen when parsing a string to a [`Sticker`].
#[derive(Debug, Error)]
#[error("Failed to parse `{0}` into `Sticker`: {1}")]
pub struct ParseStickerError(String, String);

impl FromStr for Sticker {
    type Err = ParseStickerError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        // "wt_001-v2-1"
        // "wt, 001-v2-1"
        let Some((pack, rest)) = id.split_once('_') else {
            return Err(ParseStickerError(
                id.to_string(),
                "Sticker format was not expected: expected to have `_` but did not".to_string(),
            ));
        };

        let mut parts = rest.split('-');

        let pack_number = match parts.next() {
            Some(part) => match part.parse() {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(ParseStickerError(
                        id.to_string(),
                        format!(
                            "Sticker pack number couldn't be parsed into a number: {err} `{part}`"
                        ),
                    ));
                }
            },
            None => {
                return Err(ParseStickerError(
                    id.to_string(),
                    "Sticker id doesn't have an expected pack number".to_string(),
                ));
            }
        };

        let mut version: Option<u16> = None;
        let mut id = 0;

        if let Some(part) = parts.next() {
            if part.starts_with('v') {
                version = match part.trim_start_matches('v').parse::<u16>() {
                    Ok(ok) => Some(ok),
                    Err(err) => {
                        return Err(ParseStickerError(
                            id.to_string(),
                            format!(
                                "Sticker version couldn't be parsed into a number: {err} `{part}`"
                            ),
                        ));
                    }
                };
            } else {
                id = match part.parse::<u16>() {
                    Ok(ok) => ok,
                    Err(err) => {
                        return Err(ParseStickerError(
                            id.to_string(),
                            format!("Sticker id couldn't be parsed into a number: {err} `{part}`"),
                        ));
                    }
                };
            }
        };

        if let Some(part) = parts.next() {
            id = match part.parse::<u16>() {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(ParseStickerError(
                        id.to_string(),
                        format!("Sticker id couldn't be parsed into a number: {err} `{part}`"),
                    ));
                }
            };
        }

        let sticker = Sticker {
            pack: pack.to_string(),
            pack_number,
            version,
            id,
        };

        Ok(sticker)
    }
}

/// Represents a [GIPHY](https://giphy.com) GIF.
#[derive(Debug, Clone)]
pub struct Giphy {
    id: String,
}

impl Giphy {
    /// Make a new Giphy from a Giphy id.
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Returns the Giphy id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a thumbnail quality URL for the GIF.
    pub fn thumbnail(&self) -> String {
        format!("https://media2.giphy.com/media/{}/giphy_s.gif", self.id)
    }

    /// Returns a render quality URL for the GIF.
    pub fn render(&self) -> String {
        format!("https://media1.giphy.com/media/{}/giphy.gif", self.id)
    }
}

/// Represents information about the poster of a [`Post`].
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Poster {
    webtoon: Webtoon,
    episode: u16,
    post_id: Id,
    pub(crate) cuid: Arc<str>,
    pub(crate) profile: Arc<str>,
    pub(crate) username: Arc<str>,
    pub(crate) is_creator: bool,
    pub(crate) is_blocked: bool,
    pub(crate) is_current_session_user: bool,
    pub(crate) is_current_webtoon_creator: bool,
    pub(crate) reaction: Arc<RwLock<Reaction>>,
    pub(crate) super_like: Option<u32>,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Poster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Poster")
            // omitting `webtoon`
            .field("episode", &self.episode)
            .field("cuid", &self.cuid)
            .field("profile", &self.profile)
            .field("username", &self.username)
            .field("is_creator", &self.is_creator)
            .field("is_blocked", &self.is_blocked)
            .field("is_current_session_user", &self.is_current_session_user)
            .field(
                "is_current_webtoon_creator",
                &self.is_current_webtoon_creator,
            )
            .field("reaction", &self.reaction)
            .field("super_likes", &self.super_like)
            .finish()
    }
}

impl Poster {
    /// Returns the posters `CUID`.
    ///
    /// Not to be confused with a `UUID`: [cuid2](https://github.com/paralleldrive/cuid2).
    #[must_use]
    pub fn cuid(&self) -> &str {
        &self.cuid
    }

    /// Returns the profile segment for poster in `webtoons.com/*/creator/{profile}`.
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Returns poster username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns if the session user reacted to post.
    ///
    /// Returns `true` if the user reacted, `false` if not.
    pub async fn reacted(&self) -> bool {
        let reaction = self.reaction.read().await;
        matches!(*reaction, Reaction::Upvote | Reaction::Downvote)
    }

    /// Returns if current session user is creator of post.
    ///
    /// If there is no session provided, this is always `false`.
    pub fn is_current_session_user(&self) -> bool {
        self.is_current_session_user
    }

    /// Returns if poster is a creator on the webtoons platform.
    ///
    /// This doesn't mean they are the creator of the current webtoon, just that they are a creator, though it could be of the current webtoon.
    /// For that info use [`Poster::is_current_webtoon_creator`].
    pub fn is_creator(&self) -> bool {
        self.is_creator
    }

    /// Returns if the session user is the creator of the current webtoon.
    pub fn is_current_webtoon_creator(&self) -> bool {
        self.is_current_webtoon_creator
    }

    /// Returns if the poster left a super like for the posts' episode.
    pub fn did_super_like_episode(&self) -> bool {
        self.super_like.is_some()
    }

    /// Returns the amount the poster super liked the posts' episode.
    ///
    /// Will return `None` if the poster didn't super like the episode, otherwise
    /// returns `Some` with the amount they did.
    pub fn super_like(&self) -> Option<u32> {
        self.super_like
    }

    /// Will block poster for current webtoon.
    ///
    /// Session user must be creator of the webtoon to moderate it. If this is not the case
    /// [`PosterError::InvalidPermissions`] will be returned.
    ///
    /// If attempting to block self, [`PosterError::BlockSelf`] will be returned.
    pub async fn block(&self) -> Result<(), PosterError> {
        let user = self
            .webtoon
            .client
            .get_user_info_for_webtoon(&self.webtoon)
            .await?;

        // Check first as blocking can only be done on a webtoon that user is creator of.
        if !user.is_webtoon_creator() {
            return Err(PosterError::InvalidPermissions);
        }

        if self.is_current_session_user {
            return Err(PosterError::BlockSelf);
        }

        // Return early if already blocked
        if self.is_blocked {
            return Ok(());
        }

        let page_id = format!(
            "{}_{}_{}",
            match self.webtoon.scope {
                Scope::Original(_) => "w",
                Scope::Canvas => "c",
            },
            self.webtoon.id,
            self.episode
        );

        let url = format!(
            "https://www.webtoons.com/p/api/community/v1/restriction/type/write-post/page/{page_id}/target/{}",
            self.cuid
        );

        let payload = json![
            {
                "sourcePostId": self.post_id
            }
        ];

        let token = self.webtoon.client.get_api_token().await?;

        let session = self
            .webtoon
            .client
            .session
            .as_ref()
            .ok_or(ClientError::NoSessionProvided)?;

        self.webtoon
            .client
            .http
            .post(url)
            .header("Service-Ticket-Id", "epicom")
            .header("Referer", "https://www.webtoons.com/")
            .header("Cookie", format!("NEO_SES={session}"))
            .header("Api-Token", token)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}

impl Hash for Post {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Post {}

/// Represents a reaction for a post.
///
/// These are mutually exclusive.
#[derive(Clone, Debug, Copy)]
pub enum Reaction {
    /// User has upvoted
    Upvote,
    /// User has downvoted
    Downvote,
    /// User has not voted
    None,
}

pub(super) async fn check_episode_exists(episode: &Episode) -> Result<bool, PostError> {
    let response = episode
        .webtoon
        .client
        .get_posts_for_episode(episode, None, 1)
        .await?;

    if response.status() == 404 {
        Ok(false)
    } else {
        Ok(true)
    }
}

impl IntoIterator for Posts {
    type Item = Post;

    type IntoIter = <Vec<Post> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.posts.into_iter()
    }
}

impl From<Vec<Post>> for Posts {
    fn from(value: Vec<Post>) -> Self {
        Self { posts: value }
    }
}

/// Trait providing a way to be generic over what type is returned for the word `replies`.
///
/// This was made so that `posts()` can have a single `replies` method, but provide the ability
/// to get the posts themselves as well as the count without having to come up with another name
/// for the function.
pub trait Replies: Sized + Sealed {
    /// Returns the replies for a post.
    #[allow(async_fn_in_trait, reason = "internal use only")]
    async fn replies(post: &Post) -> Result<Self, PostError>;
}

impl Sealed for u32 {}
impl Replies for u32 {
    async fn replies(post: &Post) -> Result<Self, PostError> {
        Ok(post.replies)
    }
}

impl Sealed for Posts {}
impl Replies for Posts {
    async fn replies(post: &Post) -> Result<Self, PostError> {
        // No need to make a network request when there ar no replies to fetch.
        if post.replies == 0 {
            return Ok(Posts { posts: Vec::new() });
        }
        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has a `Client` that has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut replies = HashSet::new();

        let response = post
            .episode
            .webtoon
            .client
            .get_replies_for_post(post, None, 100)
            .await?
            .text()
            .await?;

        let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

        let mut next: Option<Id> = api.result.pagination.next;

        // Add first replies
        for reply in api.result.posts {
            replies.insert(Post::try_from((&post.episode, reply))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = post
                .episode
                .webtoon
                .client
                .get_replies_for_post(post, Some(cursor), 100)
                .await?
                .text()
                .await?;

            let api = serde_json::from_str::<PostsResult>(&response).context(response)?;

            for reply in api.result.posts {
                replies.replace(Post::try_from((&post.episode, reply))?);
            }

            next = api.result.pagination.next;
        }

        let mut replies = Posts {
            posts: replies.into_iter().collect(),
        };

        replies.sort_by_oldest();

        Ok(replies)
    }
}
