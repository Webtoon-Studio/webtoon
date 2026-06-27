//! Module containing things related to posts and their posters.

use super::Episode;
use crate::{
    platform::webtoons::{Webtoon, error::PostsError, webtoon::post::id::Id},
    stdx::{
        cache::{Cache, Store},
        error::assume,
    },
};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug};
use std::{collections::HashSet, hash::Hash, str::FromStr, sync::Arc};
use thiserror::Error;

/// A single top-level comment left on an [`Episode`].
///
/// A `Comment` always refers to a **top-level** episode post. Replies to a
/// comment are represented separately as [`Reply`]'s, and can be accessed
/// via [`Comment::replies`].
///
/// # Examples
///
/// Fetching and inspecting the oldest comment on an episode:
///
/// ```
/// # use webtoon::platform::webtoons::{Client, Type, error::Error};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let webtoon = client.webtoon(9116, Type::Original).await?.expect("`9116` exists");
///
/// if let Some(episode) = webtoon.episode(11).await? {
///     let mut comments = episode.posts();
///
///     if let Some(comment) = comments.last().await? {
///         println!("{} said: {}", comment.poster().username(), comment.body().contents());
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Comment(pub(crate) Post);

impl Comment {
    /// Returns the episode number this [`Comment`] was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         assert_eq!(11, comment.episode());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn episode(&self) -> u16 {
        let comment = &self.0;
        comment.episode()
    }

    /// Returns the [`Id`] of this [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
    ///         assert_eq!(comment.id(), "GW-epicom:0-w_6054_70-1");
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn id(&self) -> Id {
        let comment = &self.0;
        comment.id()
    }

    /// Returns the [`Body`] of this [`Comment`], containing its text and spoiler flag
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(60).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("body contents: {}", comment.body().contents());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn body(&self) -> &Body {
        let comment = &self.0;
        comment.body()
    }

    /// Returns the number of upvotes on this [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("upvotes: {}", comment.upvotes());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        let comment = &self.0;
        comment.upvotes()
    }

    /// Returns the number of downvotes on this [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("downvotes: {}", comment.downvotes());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        let comment = &self.0;
        comment.downvotes()
    }

    /// Returns the upvote and downvote counts for this [`Comment`] as `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///         let (upvotes, downvotes) = comment.upvotes_and_downvotes();
    ///         println!("post has {upvotes} upvotes and {downvotes} downvotes!");
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn upvotes_and_downvotes(&self) -> (u32, u32) {
        let comment = &self.0;
        (comment.upvotes(), comment.downvotes())
    }

    /// Returns the replies on this [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(4425, Type::Original).await?.expect("`4425` exists");
    ///
    /// if let Some(episode) = webtoon.episode(87).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         for reply in comment.replies().await? {
    ///             println!("{} left a reply to {}", reply.poster().username(), comment.poster().username());
    ///         }
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    pub async fn replies(&self) -> Result<Vec<Reply>, PostsError> {
        let comment = &self.0;
        comment.replies().await
    }

    /// Returns the number of replies on this [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(4425, Type::Original).await?.expect("`4425` exists");
    ///
    /// if let Some(episode) = webtoon.episode(87).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("there are {} replies for this post!", comment.reply_count());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn reply_count(&self) -> u32 {
        let comment = &self.0;
        comment.reply_count()
    }

    /// Returns `true` if this [`Comment`] is one of the three pinned top comments on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(10).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         assert!(!comment.is_top());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_top(&self) -> bool {
        let comment = &self.0;

        let Store::Value(top_comments) = comment.episode.top_comments.get() else {
            unreachable!("`top_comments` should be cached from the initial posts request");
        };

        top_comments
            .into_iter()
            .any(|top| top.is_some_and(|top| top.id() == comment.id()))
    }

    /// Returns `true` if this [`Comment`] has been deleted.
    ///
    /// A deleted comment is only returned by [`Episode::posts()`] if it still has replies;
    /// deleted comments with no remaining replies are omitted entirely.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///         assert!(!comment.is_deleted());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        let comment = &self.0;
        comment.is_deleted()
    }

    /// Returns the Unix timestamp in milliseconds of when this [`Comment`] was posted.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("comment posted at: {}", comment.posted());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn posted(&self) -> i64 {
        let comment = &self.0;
        comment.posted()
    }

    /// Returns the [`Poster`] of this [`Comment`].
    ///
    /// With a session, the [`Poster`] includes additional metadata such as whether
    /// the comment was left by the session user.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         let poster = comment.poster();
    ///         println!("{} left a post on episode {}", poster.username(), episode.number());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn poster(&self) -> &Poster {
        let comment = &self.0;
        comment.poster()
    }
}

impl From<Post> for Comment {
    #[inline]
    fn from(post: Post) -> Self {
        Self(post)
    }
}

pub use iter::Comments;

mod iter {
    use super::{Comment, Episode, Id, Post, PostsError};
    use crate::{platform::webtoons::webtoon::post::PinRepresentation, stdx::error::assume};

    /// Internal state machine for the [`Comments`] iterator.
    ///
    /// - `Start` indicates that no requests have been made yet.
    /// - `Streaming` indicates that pages are being fetched and buffered.
    /// - `Finished` indicates that no further comments are available.
    enum State {
        Start,
        Streaming,
        Finished,
    }

    /// An asynchronous iterator over the top-level comments for an [`Episode`].
    ///
    /// Lazily fetches pages from the API and yields [`Comment`] values newest-first.
    /// Obtain one via [`Episode::posts()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(45).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///         println!("{}: {}", comment.poster().username(), comment.body().contents());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    pub struct Comments<'e> {
        episode: &'e Episode,
        buf: Vec<Post>,
        cursor: Option<Id>,
        state: State,
    }

    impl<'e> Comments<'e> {
        /// Creates a new comment iterator for the given [`Episode`].
        ///
        /// This constructor does not perform any network requests. The first
        /// request is deferred until [`Comments::next`] is called.
        #[inline]
        pub(crate) fn new(episode: &'e Episode) -> Self {
            Self {
                episode,
                // WHY:
                // The max amount of posts returned from the API at once is 100.
                // There are at most `3` pinned comments.
                buf: Vec::with_capacity(103),
                cursor: None,
                state: State::Start,
            }
        }

        /// Advances the iterator and returns the next available [`Comment`].
        ///
        /// Returns:
        /// - `Ok(Some(Comment))` when a comment is available.
        /// - `Ok(None)` when all comments have been exhausted.
        /// - `Err(PostsError)` if a network or parsing error occurs.
        pub async fn next(&mut self) -> Result<Option<Comment>, PostsError> {
            match self.state {
                State::Start => {
                    // Cache top posts.
                    {
                        let response = self
                            .episode
                            .webtoon
                            .client
                            // Gets `is_top/isPinned` info.
                            .fetch_episode_posts(
                                self.episode,
                                None,
                                10,
                                PinRepresentation::Distinct,
                            )
                            .await?;

                        assume!(
                            response.result.tops.len() < 4,
                            "there should only be at most 3 top comments on `webtoons.com` episode"
                        );

                        let mut top_comments = [None, None, None];

                        for (idx, comment) in response.result.tops.into_iter().enumerate() {
                            if let Some(top) = top_comments.get_mut(idx) {
                                *top = Some(Comment(Post::try_from((self.episode, comment))?));
                            }
                        }

                        self.episode.top_comments.insert(top_comments);
                    }

                    let response = self
                        .episode
                        .webtoon
                        .client
                        .fetch_episode_posts(self.episode, None, 100, PinRepresentation::None)
                        .await?;

                    for post in response.result.posts {
                        self.buf.push(Post::try_from((self.episode, post))?);
                    }

                    self.cursor = response.result.pagination.next;

                    self.buf.reverse();
                    self.state = State::Streaming;
                }
                State::Streaming if self.buf.is_empty() => {
                    if let Some(cursor) = self.cursor {
                        let response = self
                            .episode
                            .webtoon
                            .client
                            .fetch_episode_posts(
                                self.episode,
                                Some(cursor),
                                100,
                                PinRepresentation::None,
                            )
                            .await?;

                        for post in response.result.posts {
                            self.buf.push(Post::try_from((self.episode, post))?);
                        }

                        self.buf.reverse();
                        self.cursor = response.result.pagination.next;
                    }
                }
                State::Streaming => {}
                State::Finished => return Ok(None),
            }

            // If no comments were gotten, then finish.
            if self.buf.is_empty() {
                self.state = State::Finished;
                return Ok(None);
            }

            Ok(self.buf.pop().map(Comment))
        }

        /// Consumes the iterator and returns the oldest visible [`Comment`] on the episode, if any.
        ///
        /// Returns `Err` if an error occurs during iteration.
        pub async fn last(mut self) -> Result<Option<Comment>, PostsError> {
            let mut last = None;

            while let Some(comment) = self.next().await? {
                last = Some(comment);
            }

            Ok(last)
        }
    }
}

/// A reply to a [`Comment`] on an [`Episode`].
///
/// Replies are second-level posts always associated with a parent comment, and are
/// retrieved via [`Comment::replies()`].
///
/// # Examples
///
/// Fetching replies for the newest comment on an episode:
///
/// ```
/// # use webtoon::platform::webtoons::{Client, Type, error::Error};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
///
/// if let Some(episode) = webtoon.episode(17).await? {
///     let mut comments = episode.posts();
///
///     if let Some(comment) = comments.next().await? {
///         for reply in comment.replies().await? {
///             println!(
///                 "{} replied: {}",
///                 reply.poster().username(),
///                 reply.body().contents()
///             );
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Reply(pub(crate) Post);

impl Reply {
    /// Returns the episode number this [`Reply`] was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///        for reply in comment.replies().await? {
    ///         assert_eq!(11, reply.episode());
    ///        }
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn episode(&self) -> u16 {
        let reply = &self.0;
        reply.episode()
    }

    /// Returns the [`Id`] of this [`Reply`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             println!("id: {}", reply.id());
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn id(&self) -> Id {
        let reply = &self.0;
        reply.id()
    }

    /// Returns the [`Id`] of the parent [`Comment`] this [`Reply`] belongs to.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             println!("parent id: {}", reply.parent_id());
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn parent_id(&self) -> Id {
        let reply = &self.0;
        reply.parent_id()
    }

    /// Returns the [`Body`] of this [`Reply`], containing its text and spoiler flag.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(60).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             println!("body contents: {}", reply.body().contents());
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn body(&self) -> &Body {
        let reply = &self.0;
        reply.body()
    }

    /// Returns the number of upvotes on this [`Reply`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             let upvotes =  reply.upvotes();
    ///             println!("first reply has {upvotes} upvotes.");
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        let reply = &self.0;
        reply.upvotes()
    }

    /// Returns the number of downvotes on this [`Reply`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             let downvotes =  reply.downvotes();
    ///             println!("first reply has {downvotes} downvotes.");
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        let reply = &self.0;
        reply.downvotes()
    }

    /// Returns the upvote and downvote counts for this [`Reply`] as `(upvotes, downvotes)`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             let (upvotes, downvotes) = reply.upvotes_and_downvotes();
    ///             println!("first reply has {upvotes} upvotes and {downvotes} downvotes.");
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn upvotes_and_downvotes(&self) -> (u32, u32) {
        let reply = &self.0;
        (reply.upvotes(), reply.downvotes())
    }

    /// Returns the Unix timestamp in milliseconds of when this [`Reply`] was posted.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             println!("rpely was posted at: {}", reply.posted());
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn posted(&self) -> i64 {
        let reply = &self.0;
        reply.posted()
    }

    /// Returns the [`Poster`] of this [`Reply`].
    ///
    /// With a session, the [`Poster`] includes additional metadata such as whether
    /// the reply was left by the session user.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(2).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             println!("{} posted", reply.poster().username());
    ///             # return Ok(());
    ///        }
    ///     }
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn poster(&self) -> &Poster {
        let reply = &self.0;
        reply.poster()
    }
}

impl From<Post> for Reply {
    #[inline]
    fn from(post: Post) -> Self {
        Self(post)
    }
}

/// A post on `webtoons.com` - either a top-level comment or a reply.
#[derive(Clone)]
pub(crate) struct Post {
    pub(crate) episode: Episode,
    pub(crate) id: Id,
    pub(crate) parent_id: Id,
    pub(crate) body: Body,
    pub(crate) upvotes: u32,
    pub(crate) downvotes: u32,
    pub(crate) replies: u32,
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
            .field("is_deleted", is_deleted)
            .field("posted", posted)
            .field("poster", poster)
            .finish()
    }
}

// TODO: remove `upvotes` and `downvotes` in favor up `upvotes_and_downvotes`.
impl Post {
    #[inline]
    #[must_use]
    pub fn poster(&self) -> &Poster {
        &self.poster
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    #[inline]
    #[must_use]
    pub fn parent_id(&self) -> Id {
        self.parent_id
    }

    #[inline]
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    #[inline]
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    #[inline]
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        self.downvotes
    }

    #[expect(dead_code)]
    pub async fn is_top(&self) -> Result<bool, PostsError> {
        if let Store::Value(top_comments) = self.episode.top_comments.get() {
            Ok(top_comments
                .into_iter()
                .any(|comment| comment.is_some_and(|top| top.id() == self.id())))
        } else {
            let response = self
                .episode
                .webtoon
                .client
                // Gets `is_top/isPinned` info.
                .fetch_episode_posts(&self.episode, None, 1, PinRepresentation::Distinct)
                .await?;

            assume!(
                response.result.tops.len() < 4,
                "there should only be at most 3 top comments on `webtoons.com` episode"
            );

            let mut top_comments = [None, None, None];

            for (idx, comment) in response.result.tops.into_iter().enumerate() {
                if let Some(top) = top_comments.get_mut(idx) {
                    *top = Some(Comment(Self::try_from((&self.episode, comment))?));
                }
            }

            let is_top = top_comments
                .iter()
                .any(|comment| comment.as_ref().is_some_and(|top| top.id() == self.id()));

            self.episode.top_comments.insert(top_comments);

            Ok(is_top)
        }
    }

    #[inline]
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    #[inline]
    #[must_use]
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    #[inline]
    #[must_use]
    pub fn posted(&self) -> i64 {
        self.posted.timestamp_millis()
    }

    #[expect(
        dead_code,
        reason = "directly gets a posts upvotes and downvotes via a request and so far we just use the data initially gotten"
    )]
    pub async fn upvotes_and_downvotes(&self) -> Result<(u32, u32), PostsError> {
        let response = self
            .episode
            .webtoon
            .client
            .fetch_post_upvotes_and_downvotes(self)
            .await?;

        let mut upvotes = 0;
        let mut downvotes = 0;

        assume!(
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

    pub async fn replies(&self) -> Result<Vec<Reply>, PostsError> {
        // PERF:
        // No need to make a network request when there are no replies to fetch.
        if self.replies == 0 {
            return Ok(Vec::new());
        }

        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has a `Client` that has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut replies = HashSet::new();

        let response = self
            .episode
            .webtoon
            .client
            .fetch_replies_for_post(self, None, 100)
            .await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first replies
        for reply in response.result.posts {
            replies.insert(Self::try_from((&self.episode, reply))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .episode
                .webtoon
                .client
                .fetch_replies_for_post(self, Some(cursor), 100)
                .await?;

            for reply in response.result.posts {
                replies.replace(Self::try_from((&self.episode, reply))?);
            }

            next = response.result.pagination.next;
        }

        let replies = {
            let mut replies = replies.into_iter().map(Reply).collect::<Vec<Reply>>();
            replies.sort_unstable_by_key(|a| a.0.id);
            replies
        };

        Ok(replies)
    }

    #[inline]
    #[must_use]
    pub fn reply_count(&self) -> u32 {
        self.replies
    }
}

/// The text content of a post, along with its spoiler flag and optional flair.
#[derive(Debug, Clone)]
pub struct Body {
    pub(crate) contents: Arc<str>,
    pub(crate) is_spoiler: bool,
    pub(crate) flare: Option<Flare>,
}

impl Body {
    /// Returns the text contents of this [`Body`].
    #[inline]
    #[must_use]
    pub fn contents(&self) -> &str {
        let body = self;
        &body.contents
    }

    /// Returns the optional [`Flare`] a post can have.
    ///
    /// This can be a list of Webtoons, a single sticker, or a single giphy gif.
    #[inline]
    #[must_use]
    pub fn flare(&self) -> Option<&Flare> {
        let body = self;
        body.flare.as_ref()
    }

    /// Returns `true` if this [`Body`] is marked as a spoiler.
    #[inline]
    #[must_use]
    pub fn is_spoiler(&self) -> bool {
        let body = self;
        body.is_spoiler
    }
}

/// Extra media attached to a post: a GIF, a sticker, or a list of webtoons.
#[derive(Debug, Clone)]
pub enum Flare {
    /// A GIF in a post.
    Giphy(Giphy),
    /// A list of webtoons in a post.
    Webtoons(Vec<Webtoon>),
    /// A sticker in a post.
    Sticker(Sticker),
}

/// A sticker attached to a post.
#[derive(Debug, Clone)]
pub struct Sticker {
    pack: String,
    pack_number: u16,
    version: Option<u16>,
    id: u16,
}

impl Sticker {
    /// Returns the pack id of this [`Sticker`], e.g. `wt_001`.
    #[inline]
    #[must_use]
    pub fn pack_id(&self) -> String {
        format!("{}_{:03}", self.pack, self.pack_number)
    }

    /// Returns the full id of this [`Sticker`], e.g. `wt_001-v2-1` or `wt_001-1` if unversioned.
    #[inline]
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
        let input = id;
        let mut id = 0;

        if let Some(part) = parts.next() {
            if part.starts_with('v') {
                version = match part.trim_start_matches('v').parse::<u16>() {
                    Ok(ok) => Some(ok),
                    Err(err) => {
                        return Err(ParseStickerError(
                            input.to_string(),
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
                            input.to_string(),
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
                        input.to_string(),
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

/// A [GIPHY](https://giphy.com) GIF attached to a post.
#[derive(Debug, Clone)]
pub struct Giphy {
    id: String,
}

impl Giphy {
    /// Creates a new [`Giphy`] from a GIPHY id.
    #[inline]
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Returns the GIPHY id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a thumbnail-quality URL for this GIF.
    #[inline]
    #[must_use]
    pub fn thumbnail(&self) -> String {
        format!("https://media2.giphy.com/media/{}/giphy_s.gif", self.id)
    }

    /// Returns a full-quality URL for this GIF.
    #[inline]
    #[must_use]
    pub fn render(&self) -> String {
        format!("https://media1.giphy.com/media/{}/giphy.gif", self.id)
    }
}

/// The author of a [`Comment`] or [`Reply`].
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Poster {
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

// TODO: Can probably do better here with what is returned when some info is not known.
impl Poster {
    /// Returns the poster's [CUID](https://github.com/paralleldrive/cuid2).
    #[inline]
    #[must_use]
    pub fn cuid(&self) -> &str {
        let poster = self;
        &poster.cuid
    }

    /// Returns the profile segment for poster in `webtoons.com/en/creator/{profile}`.
    #[inline]
    #[must_use]
    pub fn profile(&self) -> &str {
        let poster = self;
        &poster.profile
    }

    /// Returns the username of this [`Poster`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?.expect("`6054` exists");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.next().await? {
    ///         println!("{} left a post on episode {}", comment.poster().username(), episode.number());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the episode block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn username(&self) -> &str {
        let poster = self;
        &poster.username
    }

    /// Returns `true` if the session user reacted to this post; `false` if not or if no session was provided.
    #[inline]
    #[must_use]
    pub fn reacted(&self) -> bool {
        let poster = self;
        matches!(
            poster.reaction.get().or_default(),
            Reaction::Upvote | Reaction::Downvote
        )
    }

    /// Returns `true` if this [`Poster`] is the current session user.
    ///
    /// Always `false` if no session was provided.
    #[inline]
    #[must_use]
    pub fn is_current_session_user(&self) -> bool {
        let poster = self;
        poster.is_current_session_user
    }

    /// Returns `true` if this [`Poster`] is a creator on the platform.
    ///
    /// This does not imply they are the creator of the current [`Webtoon`].
    #[inline]
    #[must_use]
    pub fn is_creator(&self) -> bool {
        let poster = self;
        poster.is_creator
    }

    // TODO: This should really be if the poster is the creator of the current webtoon
    // not that the session user is the current webtoon creator. Need to check what this is
    // checking.
    /// Returns `true` if the session user is the creator of the current [`Webtoon`].
    #[inline]
    #[must_use]
    pub fn is_current_webtoon_creator(&self) -> bool {
        let poster = self;
        poster.is_current_webtoon_creator
    }

    /// Returns `true` if this [`Poster`] left a super like on the episode.
    #[inline]
    #[must_use]
    pub fn did_super_like_episode(&self) -> bool {
        let poster = self;
        poster.super_like.is_some()
    }

    /// Returns the super like amount this [`Poster`] gave to the episode, if any.
    #[inline]
    #[must_use]
    pub fn super_like(&self) -> Option<u32> {
        let poster = self;
        poster.super_like
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

/// The session user's reaction to a post.
#[derive(Clone, Debug, Copy, Default)]
pub enum Reaction {
    /// The user upvoted the post.
    Upvote,
    /// The user downvoted the post.
    Downvote,
    /// The user has not reacted.
    #[default]
    None,
}

pub(crate) enum PinRepresentation {
    None,
    Distinct,
}

pub mod id {
    //! Module representing the post id format on `webtoons.com`.

    use crate::stdx::base36::Base36;
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::Ordering,
        fmt::{Debug, Display},
        num::ParseIntError,
        str::FromStr,
    };
    use thiserror::Error;

    // TODO: Make just a tuple struct that takes a `String`.
    /// Error parsing a post [`Id`].
    #[derive(Error, Debug)]
    pub enum ParsePostIdError {
        /// The id string did not match the expected format.
        #[error("failed to parse `{id}` into `Id`: {context}")]
        InvalidFormat {
            /// The original id string that failed to parse.
            id: String,
            /// A description of why parsing failed.
            context: String,
        },
        /// A numeric component of the id could not be parsed.
        #[error("failed to parse `{id}` into `Id`: {error}")]
        ParseNumber {
            /// The original id string that failed to parse.
            id: String,
            /// The underlying parse error.
            error: ParseIntError,
        },
    }

    /// A unique identifier for a post or reply on a [`Webtoon`](crate::platform::webtoons::webtoon::Webtoon) episode.
    ///
    /// IDs follow the format `GW-epicom:0-w_95_1-1d-z`, where the components encode
    /// the webtoon type, webtoon id, episode number, post position (Base36), and
    /// optionally a reply position (Base36). IDs with lower post/reply values were
    /// posted earlier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::webtoon::post::id::{Id, ParsePostIdError};
    /// # use std::str::FromStr;
    /// let id = Id::from_str("GW-epicom:0-w_95_1-1d-z")?;
    /// # Ok::<(), ParsePostIdError>(())
    /// ```
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
        type Err = ParsePostIdError;

        #[allow(clippy::too_many_lines)]
        fn from_str(str: &str) -> Result<Self, Self::Err> {
            // In some instances a string can be prefixed like `1:3595:`. We only
            // care about the part after, so if we encounter multiple `:`, we trim
            // the prefix off.
            //
            // NOTE: Its still unknown what this prefix means.
            let g = str.chars().position(|ch| ch == 'G').ok_or_else(|| {
                ParsePostIdError::InvalidFormat {
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
                .ok_or_else(|| ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "`G` in `GW-epicom` index was out of bounds, when it should never be"
                        .to_string(),
                })?
                .split(':');

            // Check that the id starts with `GW-epicom`, as this is the only known prefix.
            if let Some(prefix) = halves.next()
                && prefix != "GW-epicom"
            {
                return Err(ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: format!(
                        "splitting on `:` for an id should have a prefix of `GW-epicom` but had, `{prefix}`",
                    ),
                });
            }

            // get `0-w_95_1-1d-z`
            let id = halves
                .next()
                .ok_or_else(|| ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "there was no right-hand part after splitting on `:`".to_string(),
                })?;

            if id.is_empty() {
                return Err(ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "split on `:` resulted in expected `GW-epicom` prefix, but there was nothing to the right of it, resulting in an empty id".to_string(),
                });
            }

            // split `0-w_95_1-1d-z` to `0` `w_95_1` `1d` `z`
            let mut parts = id.split('-');

            let Some(first) = parts.next() else {
                return Err(ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "splitting on `-` should yield at least 3 parts, but trying to get the first(tag) element resulted on `None`".to_string(),
                });
            };

            let tag: u32 = first.parse().map_err(|err| ParsePostIdError::ParseNumber {
                id: str.to_owned(),
                error: err,
            })?;

            let Some(page_id) = parts.next() else {
                return Err(ParsePostIdError::InvalidFormat {
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
                    return Err(ParsePostIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should only have a scope of either `w` or `c`, but found: {s}",
                        ),
                    });
                }
                None => {
                    return Err(ParsePostIdError::InvalidFormat {
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
                        .map_err(|err| ParsePostIdError::ParseNumber {
                            id: str.to_owned(),
                            error: err,
                        })?
                }
                None => {
                    return Err(ParsePostIdError::InvalidFormat {
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
                        .map_err(|err| ParsePostIdError::ParseNumber {
                            id: str.to_owned(),
                            error: err,
                        })?
                }
                None => {
                    return Err(ParsePostIdError::InvalidFormat {
                        id: str.to_owned(),
                        context: format!(
                            r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but `{page_id}` didn't have a episode number as the third element",
                        ),
                    });
                }
            };

            if let Some(unknown) = page_id_parts.next() {
                return Err(ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: format!(
                        r"page id should consist of 3 parts, (w|c)_(\d+)_(\d+), but found `{unknown}` as part of `{page_id}`, after the expected end",
                    ),
                });
            }

            let Some(second) = parts.next() else {
                return Err(ParsePostIdError::InvalidFormat {
                    id: str.to_owned(),
                    context: "splitting on `-` should yield at least 3 parts, but trying to get the second(post number) element resulted on `None`".to_string(),
                });
            };

            // parse `1d` to `Base36`
            let post = second
                .parse::<Base36>()
                .map_err(|err| ParsePostIdError::ParseNumber {
                    id: str.to_owned(),
                    error: err,
                })?;

            // if exists parse `z` to `Base36`
            let reply = parts
                .next()
                .map(|reply| {
                    reply
                        .parse::<Base36>()
                        .map_err(|err| ParsePostIdError::ParseNumber {
                            id: str.to_owned(),
                            error: err,
                        })
                })
                .transpose()?;

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
                write!(f, "GW-epicom:{tag}-{scope}_{webtoon}_{episode}-{post}")
            }
        }
    }

    impl Debug for Id {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(&self, f)
        }
    }

    impl<'a> PartialEq<&'a str> for Id {
        fn eq(&self, other: &&'a str) -> bool {
            Self::from_str(other).is_ok_and(|id| *self == id)
        }
    }

    impl PartialEq<String> for Id {
        fn eq(&self, other: &String) -> bool {
            Self::from_str(other).is_ok_and(|id| *self == id)
        }
    }

    impl Ord for Id {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self.post.cmp(&other.post) {
                Ordering::Equal => {
                    match (self.reply, other.reply) {
                        // Both are replies to the same post.
                        (Some(reply), Some(other)) => reply.cmp(&other),
                        // A direct post is always less than a reply to it.
                        (None, Some(_)) => Ordering::Less,
                        // A reply is always greater than the direct post.
                        (Some(_), None) => Ordering::Greater,
                        // Same direct post
                        (None, None) => Ordering::Equal,
                    }
                }
                ord => ord,
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
        type Error = ParsePostIdError;

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
    mod tests {
        #![allow(clippy::unwrap_used)]

        use super::*;
        use pretty_assertions::{assert_eq, assert_ne};

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
            assert_eq!(id, "GW-epicom:0-w_95_1-1d");
            assert_eq!(id_with_reply, "GW-epicom:0-w_95_1-1d-1");
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

            assert_ne!(id, "GW-epicom:0-w_95_2-1d");
            assert_ne!(id, "GW-epicom:0-w_95_1-1d-1");
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

            assert_eq!(id.scope, Scope::W);
            assert_eq!(id.webtoon, 95);
            assert_eq!(id.episode, 1);
            assert_eq!(id.post, 49);
            assert_eq!(id.reply, None);
        }

        #[test]
        fn should_parse_reply_id() {
            {
                let id = Id::from_str("GW-epicom:0-w_95_1-1d-z").unwrap();

                assert_eq!(id.scope, Scope::W);
                assert_eq!(id.webtoon, 95);
                assert_eq!(id.episode, 1);
                assert_eq!(id.post, 49);
                assert_eq!(id.reply, Some(Base36::new(35)));
            }
            {
                let id = Id::from_str("GW-epicom:0-c_656579_161-13-1").unwrap();

                assert_eq!(id.scope, Scope::C);
                assert_eq!(id.webtoon, 656_579);
                assert_eq!(id.episode, 161);
                assert_eq!(id.post, 39);
                assert_eq!(id.reply, Some(Base36::new(1)));
            }
        }

        #[test]
        fn should_skip_unknown_prefix() {
            {
                let id = Id::from_str("`1:1300:GW-epicom:0-w_95_1-1d-z").unwrap();

                assert_eq!(id.scope, Scope::W);
                assert_eq!(id.webtoon, 95);
                assert_eq!(id.episode, 1);
                assert_eq!(id.post, 49);
                assert_eq!(id.reply, Some(Base36::new(35)));
            }
            {
                let id = Id::from_str("10:1:GW-epicom:0-c_656579_161-13-1").unwrap();

                assert_eq!(id.scope, Scope::C);
                assert_eq!(id.webtoon, 656_579);
                assert_eq!(id.episode, 161);
                assert_eq!(id.post, 39);
                assert_eq!(id.reply, Some(Base36::new(1)));
            }
        }
    }
}
