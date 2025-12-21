//! Module containing things related to posts and their posters.

use super::Episode;
use crate::{
    platform::webtoons::{Webtoon, error::PostsError, webtoon::post::id::Id},
    stdx::{
        cache::{Cache, Store},
        error::assumption,
    },
};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug};
use std::{collections::HashSet, hash::Hash, str::FromStr, sync::Arc};
use thiserror::Error;

/// A single top-level comment left on an [`Episode`].
///
/// A `Comment` always refers to a **top-level** episode post. Replies to a
/// comment are represented separately as [`Reply`] values and can be accessed
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
/// let webtoon = client.webtoon(9116, Type::Original).await?
///     .expect("webtoon exists");
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
    /// Returns the episode number the comment was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
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
        self.0.episode()
    }

    /// Returns the unique [`Id`] for the comment.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
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
        self.0.id()
    }

    /// Returns a reference to the [`Body`] of the comment.
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
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(60).await? {
    ///     let comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
    ///         assert_eq!("If Nerys is not Queen… the election is rigged", comment.body().contents());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn body(&self) -> &Body {
        self.0.body()
    }

    /// Returns how many upvotes the comment has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.upvotes()
    }

    /// Returns how many downvotes the comment has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.downvotes()
    }

    /// Returns the upvote and downvote count for the comment.
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
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        (self.upvotes(), self.downvotes())
    }

    /// Returns the replies on the current post.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(4425, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(87).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.replies().await
    }

    /// Returns the reply count for the current post.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(4425, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(87).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.reply_count()
    }

    // only has to happen once, and all other posts can just `top.any()` and
    // check.
    /// Returns whether this post is a `TOP` comment, one of the top three pinned comments on the episode.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(10).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        if let Store::Value(top_comments) = self.0.episode.top_comments.get() {
            top_comments
                .into_iter()
                .any(|comment| comment.is_some_and(|top| top.id() == self.id()))
        } else {
            unreachable!("`top_comments` should be cached from the initial posts request");
        }
    }

    /// Returns whether this reply was deleted.
    ///
    /// One thing to keep in mind is that if a comment was deleted and no replies were left,
    /// or if all replies were themselves deleted, it won't be returned in the [`Episode::posts()`](super::Episode::posts()) response.
    ///
    /// This will only return `true` if there is a comment that has replies on it. Otherwise, will return `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(1).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.is_deleted()
    }

    /// Returns the comments published date in UNIX millisecond timestamp format.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
    ///         assert_eq!(1709085249648, comment.posted());
    ///         # return Ok(());
    ///     }
    /// }
    /// # unreachable!("should have entered the post block and returned");
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn posted(&self) -> i64 {
        self.0.posted()
    }

    /// Returns the [`Poster`] of comment.
    ///
    /// If a valid session is passed to the client, this will contain some extra
    /// metadata, which can be used for determining if, for example, the comment
    /// was left by the current session user.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
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
        self.0.poster()
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
    use crate::{platform::webtoons::webtoon::post::PinRepresentation, stdx::error::assumption};

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

    /// Asynchronous iterator over the comments for a single [`Episode`].
    ///
    /// `Comments` lazily fetches comment pages from the Webtoon API and yields
    /// individual [`Comment`] values in chronological order.
    ///
    /// # Examples
    ///
    /// Iterating over comments:
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    /// .expect("webtoon exists");
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
    ///
    /// Fetch the oldest comment:
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{Client, Type, error::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    /// .expect("webtoon exists");
    ///
    /// if let Some(episode) = webtoon.episode(21).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     if let Some(comment) = comments.last().await? {
    ///         println!("last comment id: {}", comment.id());
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
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
        #[must_use]
        pub(crate) fn new(episode: &'e Episode) -> Self {
            Self {
                episode,
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
                            .episode_posts(self.episode, None, 10, PinRepresentation::Distinct)
                            .await?;

                        assumption!(
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
                        .episode_posts(self.episode, None, 100, PinRepresentation::None)
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
                            .episode_posts(self.episode, Some(cursor), 100, PinRepresentation::None)
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

        /// Consumes the iterator and returns the final [`Comment`], if any.
        ///
        /// This corresponds to the oldest comment left on the episode, that is
        /// still visible. Most commonly, this will be the first comment.
        ///
        /// # Returns
        /// - `Ok(Some(Comment))` if at least one comment exists.
        /// - `Ok(None)` if the episode has no comments.
        /// - `Err(PostsError)` if an error occurs during iteration.
        pub async fn last(mut self) -> Result<Option<Comment>, PostsError> {
            let mut last = None;

            while let Some(comment) = self.next().await? {
                last = Some(comment);
            }

            Ok(last)
        }
    }
}

/// A reply left on a [`Comment`] within an [`Episode`].
///
/// A `Reply` represents a **second-level** post in the episode discussion
/// hierarchy. Unlike [`Comment`], replies are always associated with a parent
/// comment and cannot exist independently.
///
/// Replies are retrieved through [`Comment::replies`] and are never returned
/// directly by [`Episode::posts`](super::Episode::posts).
///
/// # Relationship to `Comment`
///
/// - A [`Comment`] is a top-level post on an episode.
/// - A `Reply` is always a response to a specific comment.
/// - Each reply exposes its parent comment identifier via [`Reply::parent_id`].
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
/// let webtoon = client.webtoon(6054, Type::Original).await?
///     .expect("webtoon exists");
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
    /// Returns the episode number the reply was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
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
        self.0.episode()
    }

    /// Returns the unique [`Id`] for the reply.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!(reply.id(), "GW-epicom:0-w_6054_70-3y-1w");
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
        self.0.id()
    }

    /// Returns the parent [`Id`] of the reply.
    ///
    /// This `id` represents the current replies' parent [`Comment`].
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!(reply.parent_id(), "GW-epicom:0-w_6054_50-4n");
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
        self.0.parent_id()
    }

    /// Returns a reference to the [`Body`] of the reply.
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
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(60).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!("Her dress is hideous ", reply.body().contents());
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
        self.0.body()
    }

    /// Returns how many upvotes the reply has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!(0, reply.upvotes());
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
        self.0.upvotes()
    }

    /// Returns how many downvotes the reply has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    ///
    /// if let Some(episode) = webtoon.episode(30).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!(0, reply.downvotes());
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
        self.0.downvotes()
    }

    /// Returns the upvote and downvote count for the reply.
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
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!((0, 0), reply.upvotes_and_downvotes());
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
        (self.upvotes(), self.downvotes())
    }

    /// Returns the reply's published date in UNIX millisecond timestamp format.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    ///
    /// if let Some(episode) = webtoon.episode(11).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!(1766106029553, reply.posted());
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
        self.0.posted()
    }

    /// Returns the [`Poster`] of reply.
    ///
    /// If a valid session is passed to the client, this will contain some extra
    /// metadata, which can be used for determining if, for example, the reply
    /// was left by the current session user.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client, Type};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
    ///
    /// if let Some(episode) = webtoon.episode(70).await? {
    ///     let mut comments = episode.posts();
    ///
    ///     while let Some(comment) = comments.next().await? {
    ///        if let Some(reply) = comment.replies().await?.first() {
    ///             assert_eq!("Natsumaybe", reply.poster().username());
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
        self.0.poster()
    }
}

impl From<Post> for Reply {
    #[inline]
    fn from(post: Post) -> Self {
        Self(post)
    }
}

/// Represents a post on `webtoons.com`, either a reply or a top-level comment.
///
/// This type is not constructed directly but gotten via [`Webtoon::posts()`] or [`Episode::posts()`] and iterated through,
/// or with [`Episode::posts_for_each()`].
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

    #[allow(dead_code)]
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
                .episode_posts(&self.episode, None, 1, PinRepresentation::Distinct)
                .await?;

            assumption!(
                response.result.tops.len() < 4,
                "there should only be at most 3 top comments on `webtoons.com` episode"
            );

            let mut top_comments = [None, None, None];

            for (idx, comment) in response.result.tops.into_iter().enumerate() {
                if let Some(top) = top_comments.get_mut(idx) {
                    *top = Some(Comment(Post::try_from((&self.episode, comment))?));
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
            .post_upvotes_and_downvotes(self)
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

    pub async fn replies(&self) -> Result<Vec<Reply>, PostsError> {
        // No need to make a network request when there are
        // no replies to fetch.
        //
        // FIX: Might make this always fetch replies?
        // I know this was put in to optimize a use case, but the reply count getting
        // out-of-date is an issue.
        if self.replies == 0 {
            return Ok(Vec::new());
        }

        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has a `Client` that has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut replies = HashSet::new();

        let response = self.episode.webtoon.client.replies(self, None, 100).await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first replies
        for reply in response.result.posts {
            replies.insert(Post::try_from((&self.episode, reply))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = self
                .episode
                .webtoon
                .client
                .replies(self, Some(cursor), 100)
                .await?;

            for reply in response.result.posts {
                replies.replace(Post::try_from((&self.episode, reply))?);
            }

            next = response.result.pagination.next;
        }

        let replies = {
            let mut replies = replies.into_iter().map(Reply).collect::<Vec<Reply>>();
            replies.sort_unstable_by(|a, b| a.0.id.cmp(&b.0.id));
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
    #[inline]
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Returns the optional [`Flare`] a post can have.
    ///
    /// This can be a list of Webtoons, a single sticker, or a single giphy gif.
    #[inline]
    #[must_use]
    pub fn flare(&self) -> Option<&Flare> {
        self.flare.as_ref()
    }

    /// Returns whether this post was marked as a spoiler.
    #[inline]
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
    #[inline]
    #[must_use]
    pub fn pack_id(&self) -> String {
        format!("{}_{:03}", self.pack, self.pack_number)
    }

    /// Returns the sticker's id as a String.
    ///
    /// Example: "`wt_001-v2-1`" (version is optional: "`wt_001-1`")
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
    #[inline]
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Returns the Giphy id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns a thumbnail quality URL for the GIF.
    #[inline]
    #[must_use]
    pub fn thumbnail(&self) -> String {
        format!("https://media2.giphy.com/media/{}/giphy_s.gif", self.id)
    }

    /// Returns a render quality URL for the GIF.
    #[inline]
    #[must_use]
    pub fn render(&self) -> String {
        format!("https://media1.giphy.com/media/{}/giphy.gif", self.id)
    }
}

/// Represents information about the poster of a [`Comment`] or [`Reply`].
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

// TODO: Can probaly do better her with what is returned when some info is not known.
impl Poster {
    /// Returns the posters `CUID`.
    ///
    /// Not to be confused with a `UUID`: [cuid2](https://github.com/paralleldrive/cuid2).
    #[inline]
    #[must_use]
    pub fn cuid(&self) -> &str {
        &self.cuid
    }

    // TODO: Need to see what this returns on other languages, as not all languages
    // have profile pages, but maybe still have "profiles"?
    /// Returns the profile segment for poster in `webtoons.com/*/creator/{profile}`.
    #[inline]
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
    /// let webtoon = client.webtoon(6054, Type::Original).await?
    ///     .expect("webtoon is known to exist");
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
        &self.username
    }

    /// Returns if the session user reacted to post.
    ///
    /// Returns `true` if the user reacted, `false` if not or if no session was provided.
    #[inline]
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
    #[inline]
    #[must_use]
    pub fn is_current_session_user(&self) -> bool {
        self.is_current_session_user
    }

    /// Returns if poster is a creator on the Webtoons platform.
    ///
    /// # Note
    ///
    /// This doesn't mean they are the creator of the current Webtoon, just that they are a creator. Though it could be of the current Webtoon.
    #[inline]
    #[must_use]
    pub fn is_creator(&self) -> bool {
        self.is_creator
    }

    // TODO: This should really be if the poster is the creator of the current webtoon
    // not that the session user is the current webtoon creator. Need to check what this is
    // checking.
    /// Returns if the session user is the creator of the current webtoon.
    #[inline]
    #[must_use]
    pub fn is_current_webtoon_creator(&self) -> bool {
        self.is_current_webtoon_creator
    }

    /// Returns if the poster left a super like for the posts' episode.
    #[inline]
    #[must_use]
    pub fn did_super_like_episode(&self) -> bool {
        self.super_like.is_some()
    }

    /// Returns the amount the poster super liked the posts' episode.
    ///
    /// Will return `None` if the poster didn't super like the episode, otherwise
    /// returns `Some` with the amount they did.
    #[inline]
    #[must_use]
    pub fn super_like(&self) -> Option<u32> {
        self.super_like
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

pub mod id {
    //! Module representing the post id format on `webtoons.com`.

    use crate::stdx::base36::Base36;
    use serde::{Deserialize, Serialize};
    use std::{cmp::Ordering, fmt::Display, num::ParseIntError, str::FromStr};
    use thiserror::Error;

    // TODO: Make just a tuple struct that takes a `String`.
    /// Represents possible errors when parsing a posts id.
    #[allow(missing_docs)]
    #[derive(Error, Debug)]
    pub enum ParseIdError {
        /// Error for an invalid id format.
        #[error("failed to parse `{id}` into `Id`: {context}")]
        InvalidFormat { id: String, context: String },
        #[error("failed to parse `{id}` into `Id`: {error}")]
        ParseNumber { id: String, error: ParseIntError },
    }

    // TODO: Make generic enough to be used by `naver` as well, removing any `webtoons.com` references.
    /// Represents a unique identifier for a post or comment on a Webtoon episode.
    ///
    /// The format contains multiple components, representing a different aspect of the Webtoon, episode,
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
    /// # Example
    ///
    /// ```rust
    /// # use webtoon::platform::webtoons::webtoon::post::id::{Id, ParseIdError};
    /// # use std::str::FromStr;
    /// let id = Id::from_str("GW-epicom:0-w_95_1-1d-z")?;
    /// # Ok::<(), ParseIdError>(())
    /// ```
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
        fn from_str(str: &str) -> Result<Self, Self::Err> {
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

            // TODO: Check that is `NonZero`
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

            // TODO: Check that is `NonZero`
            // parse `1d` to `Base36`
            let post = second
                .parse::<Base36>()
                .map_err(|err| ParseIdError::ParseNumber {
                    id: str.to_owned(),
                    error: err,
                })?;

            // TODO: Check that is `NonZero`
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
