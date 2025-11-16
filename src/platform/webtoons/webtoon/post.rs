//! Module containing things related to posts and their posters.

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug};
use serde_json::json;
use std::{cmp::Ordering, collections::HashSet, hash::Hash, str::FromStr, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    platform::webtoons::{
        self, Webtoon,
        client::api::posts::Section,
        errors::{ClientError, PostError, PosterError, ReplyError},
        meta::Scope,
        webtoon::post::id::Id,
    },
    private::Sealed,
};

use super::Episode;

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

/// Represents a collection of posts.
///
/// This type is not constructed directly but gotten via [`Webtoon::posts()`] or [`Episode::posts()`].
#[derive(Debug, Clone)]
pub struct Posts {
    pub(super) posts: Vec<Post>,
}

impl Posts {
    /// Returns the first post, or `None` if it is empty.
    pub fn first(&self) -> Option<&Post> {
        self.posts.first()
    }

    /// Returns the last post, or `None` if it is empty.
    pub fn last(&self) -> Option<&Post> {
        self.posts.last()
    }

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

/// Represents a post on `webtoons.com`, either a reply or a top-level comment.
///
/// This type is not constructed directly but gotten via [`Webtoon::posts()`] or [`Episode::posts()`] and iterated through,
/// or with [`Episode::posts_for_each()`].
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

impl Debug for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            episode: _,
            id,
            parent_id,
            body,
            upvotes,
            downvotes,
            replies,
            is_top,
            is_deleted,
            posted,
            poster,
        } = self;

        f.debug_struct("Post")
            .field("id", id)
            .field("parent_id", parent_id)
            .field("body", body)
            .field("upvotes", upvotes)
            .field("downvotes", downvotes)
            .field("replies", replies)
            .field("is_top", is_top)
            .field("is_deleted", is_deleted)
            .field("posted", posted)
            .field("poster", poster)
            .finish()
    }
}

impl Post {
    /// Returns the [`Poster`] of post.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     episode.posts_for_each(async |post| {
    ///         let poster = post.poster();
    ///         println!("{} left a post on episode {}", poster.username(), episode.number());
    ///     }).await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
    pub fn poster(&self) -> &Poster {
        &self.poster
    }

    /// Returns the unique [`Id`] for the post.
    ///
    /// The returned [`Id`] contains all the necessary information to uniquely identify the post
    /// in the context of a specific Webtoon episode. This includes the Webtoon ID,
    /// episode number, post identifier, and optionally a reply identifier if the post is a reply.
    ///
    /// The [`Id`] is a composite structure that reflects the internal format used by Webtoon's system.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert_eq!(post.id(), "GW-epicom:0-w_6054_70-1");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns the parent [`Id`] of the post.
    ///
    /// If the post is a top-level comment, the parent ID will be the same as the post's own ID.
    /// If the post is a reply to another comment, the parent ID will reflect the ID of the post it is replying to.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert_eq!( post.parent_id(), "GW-epicom:0-w_6054_50-1");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parent_id(&self) -> Id {
        self.parent_id
    }

    /// Returns a reference to the [`Body`] of the post.
    ///
    /// The body contains the actual text of the post along with a flag indicating if it is marked as a spoiler.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(60).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert_eq!("If Nerys is not Queen… the election is rigged", post.body().contents());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Returns how many upvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         println!("upvotes: {}", post.upvotes());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    /// Returns how many downvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         println!("downvotes: {}", post.downvotes());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn downvotes(&self) -> u32 {
        self.downvotes
    }

    /// Returns whether this post is a top-level comment and not a reply.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert!(post.is_comment());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_comment(&self) -> bool {
        self.id == self.parent_id
    }

    /// Returns whether this post is a reply and not a top-level comment.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(20).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert!(!post.is_reply());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_reply(&self) -> bool {
        self.id != self.parent_id
    }

    /// Returns whether this post is a `TOP` post, one of the pinned top three posts on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(10).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert!(!post.is_top());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.is_top
    }

    /// Returns whether this post was deleted.
    ///
    /// One thing to keep in mind is that if a top-level post was deleted and no replies were left,
    /// or if all replies were themselves deleted, it won't be returned in the [`Episode::posts()`](super::Episode::posts()) response.
    ///
    /// This will only return `true` if there is a top-level post that has replies on it. Otherwise will return `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         assert!(!post.is_deleted());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Returns the episode number of the post was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         assert_eq!(11, post.episode());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in UNIX millisecond timestamp format.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.last() {
    ///         assert_eq!(1709085249648, post.posted());
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn posted(&self) -> i64 {
        self.posted.timestamp_millis()
    }

    /// Upvotes post via users session.
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         let (upvotes, downvotes) = post.upvote().await?;
    ///         println!("now post has {upvotes} upvotes and {downvotes} downvotes");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         let (upvotes, downvotes) = post.downvote().await?;
    ///         println!("now post has {upvotes} upvotes and {downvotes} downvotes");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         let (upvotes, downvotes) = post.unvote().await?;
    ///         println!("now post has {upvotes} upvotes and {downvotes} downvotes");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// A tuple of `(upvotes, downvotes)`
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         let (upvotes, downvotes) = post.upvotes_and_downvotes().await?;
    ///         println!("post has {upvotes} upvotes and {downvotes} downvotes!");
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upvotes_and_downvotes(&self) -> Result<(u32, u32), PostError> {
        let response = self
            .episode
            .webtoon
            .client
            .get_upvotes_and_downvotes_for_post(self)
            .await?;

        let mut upvotes = 0;
        let mut downvotes = 0;
        for emotion in response.result.emotions {
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
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type, webtoon::post::{Replies, Posts}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(4425, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(87).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         let replies: u32 = post.replies().await?;
    ///         println!("post has {replies} relies!");
    ///
    ///         let replies: Posts = post.replies().await?;
    ///
    ///         for reply in replies {
    ///             println!("{} left a reply to {}", reply.poster().username(), post.poster().username());
    ///         }
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn replies<R: Replies>(&self) -> Result<R, PostError> {
        R::replies(self).await
    }

    /// Posts a reply on top-level comment.
    ///
    /// This method allows users to leave a reply on a top-level comment. The reply can be marked as a spoiler.
    ///
    /// # Parameters:
    /// - `body`: The content of the comment to be posted.
    /// - `is_spoiler`: A boolean indicating whether the comment should be marked as a spoiler. If `true`, the comment will be marked as a spoiler.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         post.reply("Thanks for commenting!", false).await?;
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reply(&self, body: &str, is_spoiler: bool) -> Result<(), ReplyError> {
        if self.is_deleted {
            return Err(ReplyError::DeletedPost);
        }

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
    /// # Permissions
    /// **Own-post**: If the post is from the sessions user, then has permission to delete.
    /// **Webtoon-Owner**: If the current user is the creator of the Webtoon the post is on, and thus has moderation capability.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::with_session("session");
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     if let Some(post) = episode.posts().await?.first() {
    ///         post.delete().await?;
    ///         # return Ok(());
    ///     }
    /// # unreachable!("should have entered the post block and returned");
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

impl TryFrom<(&Episode, webtoons::client::api::posts::RawPost)> for Post {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(
        (episode, post): (&Episode, webtoons::client::api::posts::RawPost),
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

impl Debug for Poster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            webtoon: _,
            episode,
            post_id,
            cuid,
            profile,
            username,
            is_creator,
            is_blocked,
            is_current_session_user,
            is_current_webtoon_creator,
            reaction,
            super_like,
        } = self;

        f.debug_struct("Poster")
            .field("episode", episode)
            .field("post_id", post_id)
            .field("cuid", cuid)
            .field("profile", profile)
            .field("username", username)
            .field("is_creator", is_creator)
            .field("is_blocked", is_blocked)
            .field("is_current_session_user", is_current_session_user)
            .field("is_current_webtoon_creator", is_current_webtoon_creator)
            .field("reaction", reaction)
            .field("super_likes", super_like)
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
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{errors::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(6054, Type::Original).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     episode.posts_for_each(async |post| {
    ///         let poster = post.poster();
    ///         println!("{} left a post on episode {}", poster.username(), episode.number());
    ///     }).await?;
    ///     # return Ok(());
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # Ok(())
    /// # }
    /// ```
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
/// <div class="warning">
///
/// **These are mutually exclusive**
///
/// </div>
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
    let status_code = episode
        .webtoon
        .client
        .get_status_code_for_episode(episode, None, 1)
        .await?;

    Ok(status_code != 404)
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
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first replies
        for reply in response.result.posts {
            replies.insert(Post::try_from((&post.episode, reply))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = post
                .episode
                .webtoon
                .client
                .get_replies_for_post(post, Some(cursor), 100)
                .await?;

            for reply in response.result.posts {
                replies.replace(Post::try_from((&post.episode, reply))?);
            }

            next = response.result.pagination.next;
        }

        let mut replies = Posts {
            posts: replies.into_iter().collect(),
        };

        replies.sort_by_oldest();

        Ok(replies)
    }
}

pub(crate) mod id {
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
    /// `GW-epicom:0-w_95_1-1d-z`
    ///
    /// - **`GW-epicom`**:
    ///   This prefix can be ignored and seems to serve as a namespace. `epicom` stands for "episode comment."
    ///
    /// - **`0`**:
    ///   This is an unknown tag. Its purpose remains unclear, but it is preserved in the ID structure for compatibility.
    ///
    /// - **`w` / `c`**:
    ///   This denotes whether the Webtoon is an **Original** (`w`) or **Canvas** (`c`).
    ///
    /// - **`95`**:
    ///   Represents the Webtoon ID. This value is unique to the Webtoon series.
    ///
    /// - **`1`**:
    ///   Represents the episode number within the Webtoon series.
    ///
    /// - **`1d`**:
    ///   A unique identifier for the specific post. It is encoded in **Base36** (using characters `0-9` and `a-z`).
    ///   This value indicates the chronological order of the post within the episode's comments section. Posts and replies cannot have a value of `0`.
    ///
    /// - **`z`**:
    ///   Represents a reply to a post. If this component is missing, the ID refers to a top-level post. If present, it indicates the reply to a specific post, also encoded in **Base36**.
    ///
    /// ### Fields:
    ///
    /// - `tag`:
    ///   An unknown field that is part of the ID structure but its exact purpose is not fully understood. It is included for completeness.
    ///
    /// - `scope`:
    ///   A string representing whether the Webtoon is an **Original** or **Canvas** series (`w` or `c`).
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
            // split `GW-epicom:0-w_95_1-1d-z` to GW-epicom` and `0-w_95_1-1d-z`
            let id = s
                .split(':')
                // get `0-w_95_1-1d-z`
                .next_back()
                .ok_or_else(|| ParseIdError::InvalidFormat {
                    id: s.to_owned(),
                    context: "there was no right-hand part after splitting on `:`".to_string(),
                })?;

            // split `0-w_95_1-1d-z` to `0` `w_95_1` `1d` `z`
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
            // split `w_95_1` to `w` `95` `1`
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

            // trick to get a static str from a runtime value
            let scope = match page_id_parts[0] {
                "w" => Scope::W,
                "c" => Scope::C,
                _ => unreachable!("a webtoon can only be either an original or canvas"),
            };

            // parse `95` to u32
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

            // parse `1d` to `Base36`
            let post = parts[2].parse().map_err(|err| ParseIdError::ParseNumber {
                id: s.to_owned(),
                error: err,
            })?;

            // if exists parse `z` to `Base36`
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
    pub enum Scope {
        W,
        C,
    }

    impl Display for Scope {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let letter = match self {
                Self::W => "w",
                Self::C => "c",
            };

            write!(f, "{letter}")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn should_be_equal_str() {
            let id = Id {
                tag: 0,
                scope: Scope::W,
                webtoon: 95,
                episode: 1,
                post: Base36::new(49),
                reply: None,
            };

            let id_with_reply = Id {
                tag: 0,
                scope: Scope::W,
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
                scope: Scope::W,
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
                scope: Scope::W,
                webtoon: 95,
                episode: 1,
                post: Base36::new(49),
                reply: None,
            };

            let fifty = Id {
                tag: 0,
                scope: Scope::W,
                webtoon: 95,
                episode: 1,
                post: Base36::new(50),
                reply: None,
            };

            let fifty_with_reply = Id {
                tag: 0,
                scope: Scope::W,
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
                scope: Scope::W,
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
                scope: Scope::C,
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

            pretty_assertions::assert_eq!(id.scope, Scope::W);
            pretty_assertions::assert_eq!(id.webtoon, 95);
            pretty_assertions::assert_eq!(id.episode, 1);
            pretty_assertions::assert_eq!(id.post, 49);
            pretty_assertions::assert_eq!(id.reply, None);
        }

        #[test]
        fn should_parse_reply_id() {
            {
                let id = Id::from_str("GW-epicom:0-w_95_1-1d-z").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::W);
                pretty_assertions::assert_eq!(id.webtoon, 95);
                pretty_assertions::assert_eq!(id.episode, 1);
                pretty_assertions::assert_eq!(id.post, 49);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(35)));
            }
            {
                let id = Id::from_str("GW-epicom:0-c_656579_161-13-1").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::C);
                pretty_assertions::assert_eq!(id.webtoon, 656_579);
                pretty_assertions::assert_eq!(id.episode, 161);
                pretty_assertions::assert_eq!(id.post, 39);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(1)));
            }
        }
    }
}
