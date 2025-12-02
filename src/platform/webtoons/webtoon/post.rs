//! Module containing things related to posts and their posters.

use chrono::{DateTime, Utc};
use core::fmt::{self, Debug};
use std::{cmp::Ordering, collections::HashSet, hash::Hash, str::FromStr, sync::Arc};
use thiserror::Error;

use crate::{
    platform::webtoons::{
        Webtoon,
        error::{BlockUserError, PostError, PostsError, ReplyError},
        webtoon::post::id::Id,
    },
    private::Sealed,
    stdx::{cache::Cache, error::assumption},
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
    #[must_use]
    pub fn first(&self) -> Option<&Post> {
        self.posts.first()
    }

    /// Returns the last post, or `None` if it is empty.
    #[must_use]
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Returns how many upvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    /// Returns how many downvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        self.downvotes
    }

    /// Returns whether this post is a top-level comment and not a reply.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn is_comment(&self) -> bool {
        self.id == self.parent_id
    }

    /// Returns whether this post is a reply and not a top-level comment.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.id != self.parent_id
    }

    /// Returns whether this post is a `TOP` post, one of the pinned top three posts on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Returns the episode number of the post was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in UNIX millisecond timestamp format.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
        // If true, user is trying to upvote their own post, which is not allowed.
        if self.poster.is_current_session_user {
            // TODO: return Err(UpvoteError::UpvoteSelf);
            return self.upvotes_and_downvotes().await;
        }

        match self.poster.reaction.get().or_default() {
            // Already upvoted the post, return with current values.
            Reaction::Upvote => {
                return self.upvotes_and_downvotes().await;
            }
            // If current reaction is `downvote`, then must unvote before
            // we can upvote.
            Reaction::Downvote => {
                self.unvote().await?;
            }
            Reaction::None => {}
        }

        self.episode
            .webtoon
            .client
            .react_to_post(self, Reaction::Upvote)
            .await?;

        // TODO: Confirm that it actually changed.

        // Set internal representation to `upvote`.
        self.poster.reaction.insert(Reaction::Upvote);

        self.upvotes_and_downvotes().await
    }

    /// Downvotes post via users session.
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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

        match self.poster.reaction.get().or_default() {
            // Must first remove upvote before we can downvote.
            Reaction::Upvote => {
                self.unvote().await?;
            }
            // Already downvoted the post, return with current values.
            Reaction::Downvote => {
                return self.upvotes_and_downvotes().await;
            }
            Reaction::None => {}
        }

        self.episode
            .webtoon
            .client
            .react_to_post(self, Reaction::Downvote)
            .await?;

        // TODO: Confirm that it actually changed.

        // Set internal representation to `downvote`.
        self.poster.reaction.insert(Reaction::Downvote);

        self.upvotes_and_downvotes().await
    }

    /// Will clear any upvote or downvote the user might have on the post.
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
        match self.poster.reaction.get().or_default() {
            Reaction::Upvote => {
                self.episode
                    .webtoon
                    .client
                    .remove_post_reaction(self, Reaction::Upvote)
                    .await?;
            }
            Reaction::Downvote => {
                self.episode
                    .webtoon
                    .client
                    .remove_post_reaction(self, Reaction::Downvote)
                    .await?;
            }
            Reaction::None => {}
        }

        self.poster.reaction.insert(Reaction::None);

        // Get updated values.
        self.upvotes_and_downvotes().await
    }

    /// Returns the upvote and downvote count for the post.
    ///
    /// A tuple of `(upvotes, downvotes)`
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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

        assumption!(
            response.result.emotions.len() < 3,
            "`webtoons.com` post api should only have either upvotes or downvotes, yet had three items: {:?}",
            response.result.emotions
        );

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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type, webtoon::post::{Replies, Posts}};
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
    pub async fn replies<R: Replies>(&self) -> Result<R, PostsError> {
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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

/// Represents the body of a post, including its content and whether it is marked as a spoiler.
///
/// The body contains the text content of the post, and a flag indicating whether the post contains spoilers.
#[derive(Debug, Clone)]
pub struct Body {
    pub(crate) contents: Arc<str>,
    pub(crate) is_spoiler: bool,
    pub(crate) flare: Option<Flare>,
}

impl Body {
    /// Returns contents of the post body.
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Returns the optional [`Flare`] a post can have.
    ///
    /// This can be a list of Webtoons, a single sticker, or a single giphy gif.
    #[must_use]
    pub fn flare(&self) -> Option<&Flare> {
        self.flare.as_ref()
    }

    /// Returns whether this post was marked as a spoiler.
    #[must_use]
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
    /// A list of Webtoons in a post.
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
    #[must_use]
    pub fn pack_id(&self) -> String {
        format!("{}_{:03}", self.pack, self.pack_number)
    }

    /// Returns the sticker's id as a String.
    ///
    /// Example: "`wt_001-v2-1`" (version is optional: "`wt_001-1`")
    #[must_use]
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
        }

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

        let sticker = Self {
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
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Returns the Giphy id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a thumbnail quality URL for the GIF.
    #[must_use]
    pub fn thumbnail(&self) -> String {
        format!("https://media2.giphy.com/media/{}/giphy_s.gif", self.id)
    }

    /// Returns a render quality URL for the GIF.
    #[must_use]
    pub fn render(&self) -> String {
        format!("https://media1.giphy.com/media/{}/giphy.gif", self.id)
    }
}

/// Represents information about the poster of a [`Post`].
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Poster {
    pub(crate) webtoon: Webtoon,
    pub(crate) episode: u16,
    pub(crate) post_id: Id,
    pub(crate) cuid: Arc<str>,
    pub(crate) profile: Arc<str>,
    pub(crate) username: Arc<str>,
    pub(crate) is_creator: bool,
    pub(crate) is_blocked: bool,
    pub(crate) is_current_session_user: bool,
    pub(crate) is_current_webtoon_creator: bool,
    pub(crate) reaction: Cache<Reaction>,
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
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
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
    #[must_use]
    pub fn reacted(&self) -> bool {
        matches!(
            self.reaction.get().or_default(),
            Reaction::Upvote | Reaction::Downvote
        )
    }

    /// Returns if current session user is creator of post.
    ///
    /// If there is no session provided, this is always `false`.
    #[must_use]
    pub fn is_current_session_user(&self) -> bool {
        self.is_current_session_user
    }

    /// Returns if poster is a creator on the Webtoons platform.
    ///
    /// This doesn't mean they are the creator of the current Webtoon, just that they are a creator, though it could be of the current Webtoon.
    /// For that info use [`Poster::is_current_webtoon_creator`].
    #[must_use]
    pub fn is_creator(&self) -> bool {
        self.is_creator
    }

    /// Returns if the session user is the creator of the current webtoon.
    #[must_use]
    pub fn is_current_webtoon_creator(&self) -> bool {
        self.is_current_webtoon_creator
    }

    /// Returns if the poster left a super like for the posts' episode.
    #[must_use]
    pub fn did_super_like_episode(&self) -> bool {
        self.super_like.is_some()
    }

    /// Returns the amount the poster super liked the posts' episode.
    ///
    /// Will return `None` if the poster didn't super like the episode, otherwise
    /// returns `Some` with the amount they did.
    #[must_use]
    pub fn super_like(&self) -> Option<u32> {
        self.super_like
    }

    /// Will block poster for current webtoon.
    ///
    /// Session user must be creator of the webtoon to moderate it. If this is not the case
    /// [`PosterError::InvalidPermissions`] will be returned.
    ///
    /// If attempting to block self, [`PosterError::BlockSelf`] will be returned.
    pub async fn block(&self) -> Result<(), BlockUserError> {
        // Return early if already blocked.
        if self.is_blocked {
            return Ok(());
        }

        if self.is_current_session_user {
            return Err(BlockUserError::BlockSelf);
        }

        let user = self
            .webtoon
            .client
            .get_user_info_for_webtoon(&self.webtoon)
            .await?;

        // Blocking can only be done on a Webtoon that user is creator of.
        if !user.is_webtoon_creator() {
            return Err(BlockUserError::NotCreator);
        }

        self.webtoon.client.block_user(self).await?;

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
#[derive(Clone, Debug, Copy, Default)]
pub enum Reaction {
    /// User has upvoted
    Upvote,
    /// User has downvoted
    Downvote,
    /// User has not voted
    #[default]
    None,
}

pub(crate) enum PinRepresentation {
    None,
    Distinct,
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
    async fn replies(post: &Post) -> Result<Self, PostsError>;
}

impl Replies for u32 {
    async fn replies(post: &Post) -> Result<Self, PostsError> {
        // FIX: While this is fast and gotten from the initial request, this can
        // also become out-of-date if a `post.reply()` is used followed by a
        // `post.replies()`.
        //
        // No matter which `replies` is used, both `post.replies` either to
        // return directly, or to check if `0`, which as this does not update,
        // if checked after a `reply()`, then it would be `0` without a full
        // re-scrape of the posts again.
        //
        // The tricky part here is when there is no need to update the reply count
        // yet wanting to optimize for a:
        //
        // ```rust
        // for post in webtoon.posts().await {
        //     let replies = post.replies::<u32>();
        // }
        // ```
        //
        // Use case, with no extra network round-trip.
        Ok(post.replies)
    }
}

impl Sealed for Posts {}
impl Replies for Posts {
    async fn replies(post: &Post) -> Result<Self, PostsError> {
        // No need to make a network request when there are
        // no replies to fetch.
        //
        // FIX: Might make this always fetch replies?
        // I know this was put in to optimize a use case, but the reply count getting
        // out-of-date is an issue.
        if post.replies == 0 {
            return Ok(Self { posts: Vec::new() });
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

        let mut replies = Self {
            posts: replies.into_iter().collect(),
        };

        replies.sort_by_oldest();

        Ok(replies)
    }
}

pub(crate) mod id {
    use crate::stdx::base36::Base36;
    use serde::{Deserialize, Serialize};
    use std::{cmp::Ordering, fmt::Display, num::ParseIntError, str::FromStr};
    use thiserror::Error;

    type Result<T, E = ParseIdError> = core::result::Result<T, E>;

    /// Represents possible errors when parsing a posts id.
    #[derive(Error, Debug)]
    pub enum ParseIdError {
        /// Error for an invalid id format.
        #[error("failed to parse `{id}` into `Id`: {context}")]
        InvalidFormat { id: String, context: String },
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

        #[allow(clippy::too_many_lines)]
        fn from_str(str: &str) -> Result<Self> {
            // In some instances a string can be prefixed like `1:3595:`. We only
            // care about the part after, so if we encounter multiple `:`, we trim
            // the prefix off.
            //
            // NOTE: Its still unknown what this prefix means.
            let g = str.chars().position(|ch| ch == 'G').ok_or_else(|| {
                ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context:
                        "a `G` should always exist within a posts id, even if it just that it starts with `GW`"
                            .to_string(),
                }
            })?;

            // Split `GW-epicom:0-w_95_1-1d-z` to `GW-epicom` and `0-w_95_1-1d-z`.
            let mut halves = str
                // In slice so that we always start with `GW-epicom`.
                .get(g..)
                .ok_or_else(|| ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "`G` in `GW-epicom` index was out of bounds, when it should never be"
                        .to_string(),
                })?
                .split(':');

            // Check that the id starts with `GW-epicom`, as this is the only known prefix.
            if let Some(prefix) = halves.next()
                && prefix != "GW-epicom"
            {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: format!(
                        "splitting on `:` for an id should have a prefix of `GW-epicom` but had, `{prefix}`",
                    ),
                });
            }

            // get `0-w_95_1-1d-z`
            let id = halves.next().ok_or_else(|| ParseIdError::InvalidFormat {
                id: str.to_owned(),
                context: "there was no right-hand part after splitting on `:`".to_string(),
            })?;

            if id.is_empty() {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "split on `:` resulted in expected `GW-epicom` prefix, but there was nothing to the right of it, resulting in an empty id".to_string(),
                });
            }

            // split `0-w_95_1-1d-z` to `0` `w_95_1` `1d` `z`
            let mut parts = id.split('-');

            let Some(first) = parts.next() else {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "splitting on `-` should yield at least 3 parts, but trying to get the first(tag) element resulted on `None`".to_string(),
                });
            };

            let tag: u32 = first.parse().map_err(|err| ParseIdError::ParseNumber {
                id: str.to_owned(),
                error: err,
            })?;

            let Some(page_id) = parts.next() else {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "splitting on `-` should yield at least 3 parts, but trying to get the second(page id) element resulted on `None`".to_string(),
                });
            };

            // split `w_95_1` to `w` `95` `1`
            let mut page_id_parts = page_id.split('_');

            let scope = match page_id_parts.next() {
                Some("w") => Scope::W,
                Some("c") => Scope::C,
                Some(s) => {
                    return Err(ParseIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should only have a scope of either `w` or `c`, but found: {s}",
                        ),
                    });
                }
                None => {
                    return Err(ParseIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but `{page_id}` didn't have a scope as the first element",
                        ),
                    });
                }
            };

            // parse `95` to `u32`
            let webtoon = match page_id_parts.next() {
                Some(webtoon) => {
                    webtoon
                        .parse::<u32>()
                        .map_err(|err| ParseIdError::ParseNumber {
                            id: str.to_owned(),
                            error: err,
                        })?
                }
                None => {
                    return Err(ParseIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but `{page_id}` didn't have a webtoon id as the second element",
                        ),
                    });
                }
            };

            // parse `1` to u16
            let episode = match page_id_parts.next() {
                Some(episode) => {
                    episode
                        .parse::<u16>()
                        .map_err(|err| ParseIdError::ParseNumber {
                            id: str.to_owned(),
                            error: err,
                        })?
                }
                None => {
                    return Err(ParseIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but `{page_id}` didn't have a episode number as the third element",
                        ),
                    });
                }
            };

            if let Some(unknown) = page_id_parts.next() {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: format!(
                        r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but found `{unknown}` as part of `{page_id}`, after the expected end",
                    ),
                });
            }

            let Some(second) = parts.next() else {
                return Err(ParseIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "splitting on `-` should yield at least 3 parts, but trying to get the second(post number) element resulted on `None`".to_string(),
                });
            };

            // parse `1d` to `Base36`
            let post = second
                .parse::<Base36>()
                .map_err(|err| ParseIdError::ParseNumber {
                    id: str.to_owned(),
                    error: err,
                })?;

            // if exists parse `z` to `Base36`
            let reply = match parts.next() {
                Some(reply) => {
                    Some(
                        reply
                            .parse::<Base36>()
                            .map_err(|err| ParseIdError::ParseNumber {
                                id: str.to_owned(),
                                error: err,
                            })?,
                    )
                }
                None => None,
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
            let Self {
                tag,
                scope,
                webtoon,
                episode,
                post,
                reply,
            } = self;

            if let Some(reply) = reply {
                write!(
                    f,
                    "GW-epicom:{tag}-{scope}_{webtoon}_{episode}-{post}-{reply}"
                )
            } else {
                write!(f, "GW-epicom:{tag}-{scope}_{webtoon}_{episode}-{post}",)
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

                        // If there is no reply number for the first one, it must be a direct post. If there is any
                        // id that has a reply with a matching post number, it must always be `Greater`, and therefore
                        // `self` must be `Less` than the reply.
                        (None, Some(_)) => Ordering::Less,

                        // Inverse of the above: If there is a reply for the first one, and the `Rhs` is None(a direct post)
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
            // checks are enough to know if the post is on the same Webtoon and the same episode it should be fine.
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
        #![allow(clippy::unwrap_used)]

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

        #[test]
        fn should_skip_unknown_prefix() {
            {
                let id = Id::from_str("`1:1300:GW-epicom:0-w_95_1-1d-z").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::W);
                pretty_assertions::assert_eq!(id.webtoon, 95);
                pretty_assertions::assert_eq!(id.episode, 1);
                pretty_assertions::assert_eq!(id.post, 49);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(35)));
            }
            {
                let id = Id::from_str("10:1:GW-epicom:0-c_656579_161-13-1").unwrap();

                pretty_assertions::assert_eq!(id.scope, Scope::C);
                pretty_assertions::assert_eq!(id.webtoon, 656_579);
                pretty_assertions::assert_eq!(id.episode, 161);
                pretty_assertions::assert_eq!(id.post, 39);
                pretty_assertions::assert_eq!(id.reply, Some(Base36::new(1)));
            }
        }
    }
}
