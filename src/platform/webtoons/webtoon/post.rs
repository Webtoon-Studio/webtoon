//! Module containing things related to posts and their posters.

use super::Episode;
use crate::{
    platform::webtoons::{Webtoon, error::PostsError},
    stdx::cache::{Cache, Store},
};
use assumptions::{Assume, Assumption, assume_matches};
use chrono::{DateTime, Utc};
use core::fmt::{self, Debug};
use id::Id;
use std::{collections::HashSet, hash::Hash, str::FromStr, sync::Arc};

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
        let comment = &self.0.id();
        let episode = &self.0.episode;

        let Store::Value(top_comments) = episode.top_comments.get() else {
            unreachable!("`top_comments` should be cached from the initial posts' request");
        };

        top_comments.contains(comment)
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
    use std::collections::VecDeque;

    use arrayvec::ArrayVec;
    use assumptions::assume;

    use super::{Comment, Episode, Id, PinRepresentation, Post, PostsError};

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
        posts: VecDeque<Post>,
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
                // MAGIC: `100`: The max amount of posts returned from the API at once is 100.
                posts: VecDeque::with_capacity(100),
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
            let iter = self;

            let episode = iter.episode;
            let client = &iter.episode.webtoon.client;
            let posts = &mut iter.posts;

            match iter.state {
                State::Finished => return Ok(None),
                State::Start => {
                    let response = client
                        // MAGIC: `Distinct`: includes `is_top/isPinned` metadata.
                        .fetch_episode_posts(episode, None, 10, PinRepresentation::Distinct)
                        .await?;

                    let count = response.result.tops.len();

                    assume!(
                        count < 4,
                        "there should only be at most 3 top comments on `webtoons.com` episode"
                    );

                    let mut top_comments = ArrayVec::new();

                    for post in response.result.tops {
                        let top_comment = Post::try_from((episode, post))?;
                        top_comments.push(top_comment.id());
                    }

                    debug_assert_eq!(
                        top_comments.len(),
                        count,
                        "all top comments from the response should have been pushed"
                    );

                    episode.top_comments.insert(top_comments);

                    // Fetch the first page of comments.
                    let response = client
                        .fetch_episode_posts(episode, None, 100, PinRepresentation::None)
                        .await?;

                    for post in response.result.posts {
                        posts.push_back(Post::try_from((episode, post))?);
                    }

                    iter.cursor = response.result.pagination.next;

                    iter.state = State::Streaming;
                }
                State::Streaming if posts.is_empty() => {
                    if let Some(cursor) = iter.cursor {
                        let response = client
                            .fetch_episode_posts(
                                episode,
                                Some(cursor),
                                100,
                                PinRepresentation::None,
                            )
                            .await?;

                        for post in response.result.posts {
                            posts.push_back(Post::try_from((episode, post))?);
                        }

                        iter.cursor = response.result.pagination.next;
                    }
                }
                State::Streaming => {}
            }

            if posts.is_empty() {
                iter.state = State::Finished;
                return Ok(None);
            }

            Ok(posts.pop_front().map(Comment))
        }

        /// Consumes the iterator and returns the oldest visible [`Comment`] on the episode, if any.
        ///
        /// Returns `Err` if an error occurs during iteration.
        pub async fn last(self) -> Result<Option<Comment>, PostsError> {
            let mut iter = self;

            let mut last = None;

            while let Some(comment) = iter.next().await? {
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
        let post = self;
        &post.poster
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> Id {
        let post = self;
        post.id
    }

    #[inline]
    #[must_use]
    pub fn parent_id(&self) -> Id {
        let post = self;
        post.parent_id
    }

    #[inline]
    #[must_use]
    pub fn body(&self) -> &Body {
        let post = self;
        &post.body
    }

    #[inline]
    #[must_use]
    pub fn upvotes(&self) -> u32 {
        let post = self;
        post.upvotes
    }

    #[inline]
    #[must_use]
    pub fn downvotes(&self) -> u32 {
        let post = self;
        post.downvotes
    }

    #[inline]
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        let post = self;
        post.is_deleted
    }

    #[inline]
    #[must_use]
    pub fn episode(&self) -> u16 {
        let post = self;
        post.episode.number()
    }

    #[inline]
    #[must_use]
    pub fn posted(&self) -> i64 {
        let post = self;
        post.posted.timestamp_millis()
    }

    pub async fn replies(&self) -> Result<Vec<Reply>, PostsError> {
        let post = self;
        let episode = &self.episode;
        let client = &self.episode.webtoon.client;

        // PERF:
        // No need to make a network request when there are no replies to fetch.
        if post.replies == 0 {
            return Ok(Vec::new());
        }

        #[allow(
            clippy::mutable_key_type,
            reason = "`Post` has a `Client` that has interior mutability, but the `Hash` implementation only uses an id: Id, which has no mutability"
        )]
        let mut replies = HashSet::new();

        let response = client.fetch_replies_for_post(post, None, 100).await?;

        let mut next: Option<Id> = response.result.pagination.next;

        // Add first replies
        for reply in response.result.posts {
            replies.insert(Self::try_from((episode, reply))?);
        }

        // Get rest if any
        while let Some(cursor) = next {
            let response = client
                .fetch_replies_for_post(post, Some(cursor), 100)
                .await?;

            for reply in response.result.posts {
                replies.replace(Self::try_from((episode, reply))?);
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
        let post = self;
        post.replies
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
        let sticker = self;
        match sticker.version {
            Some(version) => {
                format!(
                    "{}_{:03}-v{version}-{}",
                    sticker.pack, sticker.pack_number, sticker.id
                )
            }
            None => {
                format!("{}_{:03}-{}", sticker.pack, sticker.pack_number, sticker.id)
            }
        }
    }
}

impl FromStr for Sticker {
    type Err = Assumption;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        // "wt_001-v2-1" -> (`wt`, `001-v2-1`)
        let (pack, rest) = id
            .split_once('_')
            .assumption("sticker id should contain `_` separating pack name from the rest")?;

        let mut parts = rest.split('-');

        let pack_number = parts
            .next()
            .assumption("sticker id should have a pack number after `_`")?
            .parse()
            .assumption("sticker pack number should be a valid `u16`")?;

        let next = parts
            .next()
            .assumption("sticker id should have at least one more part after the pack number")?;

        let (version, id) = match next {
            v if v.starts_with('v') => {
                let version = v
                    .trim_start_matches('v')
                    .parse::<u16>()
                    .assumption("sticker version should be a valid `u16`")?;
                let sticker_id = parts
                    .next()
                    .assumption("sticker id should have an id part after the version")?
                    .parse::<u16>()
                    .assumption("sticker id should be a valid `u16`")?;
                (Some(version), sticker_id)
            }
            id => {
                let sticker_id = id
                    .parse::<u16>()
                    .assumption("sticker id should be a valid `u16`")?;
                (None, sticker_id)
            }
        };

        assume_matches!(
            parts.next(),
            None,
            "all parts of sticker id should have been consumed"
        );

        let sticker = Self {
            pack: pack.to_owned(),
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
    /// Do not include top comments in API response.
    None,
    /// To include top comments in API response.
    Distinct,
}

pub mod id {
    //! Module representing the post id format on `webtoons.com`.

    use crate::stdx::base36::Base36;
    use assumptions::{Assume, Assumption, assume_matches, assumption};
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::Ordering,
        debug_assert_matches,
        fmt::{Debug, Display},
        str::FromStr,
    };

    /// A unique identifier for a post or reply on a [`Webtoon`](crate::platform::webtoons::webtoon::Webtoon) episode.
    ///
    /// IDs follow the format `GW-epicom:0-w_95_1-1d-z`, where the components encode
    /// the webtoon type, webtoon id, episode number, post position (Base36), and
    /// optionally a reply position (Base36). IDs with lower post/reply values were
    /// posted earlier.
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
        type Err = Assumption;

        #[inline]
        fn from_str(str: &str) -> Result<Self, Self::Err> {
            #[derive(Debug)]
            enum Parse {
                Tag,
                PageId,
                Post,
                Reply,
            }

            let id = match str.split_once("GW-epicom") {
                Some((_, id)) if !id.is_empty() => id.trim_start_matches(':'),
                Some(_) => assumption!(
                    "splitting on `GW-epicom` should yield a suffix that is never empty: `{str}`"
                ),
                None => assumption!("`GW-epicom` should always be part of a posts' id: `{str}`"),
            };

            let mut tag = None;
            let mut scope = None;
            let mut webtoon = None;
            let mut episode = None;
            let mut post = None;
            let mut reply = None;

            let mut state = Parse::Tag;

            // split `0-w_95_1-1d-z` to `0` `w_95_1` `1d` `z`
            for part in id.split('-') {
                match state {
                    Parse::Tag => {
                        tag = Some(part.parse::<u32>().assumption("")?);
                        state = Parse::PageId;
                    }
                    Parse::PageId => {
                        // split `w_95_1` to `w` `95` `1`
                        let mut page_id = part.split('_');

                        scope = match page_id.next() {
                            Some("w") => Some(Scope::W),
                            Some("c") => Some(Scope::C),
                            Some(s) => assumption!("should be `w` or `c`, found: {s}"),
                            None => assumption!("page id should consist of 3 parts: {part}"),
                        };

                        webtoon = match page_id.next() {
                            Some(webtoon) => Some(webtoon.parse::<u32>().assumption("")?),
                            None => assumption!("page id should consist of 3 parts: {part}"),
                        };

                        episode = match page_id.next() {
                            Some(episode) => Some(episode.parse::<u16>().assumption("")?),
                            None => assumption!("page id should consist of 3 parts: {part}"),
                        };

                        assume_matches!(
                            page_id.next(),
                            None,
                            "`page_id` should only have 3 parts: {part}"
                        );

                        state = Parse::Post;
                    }
                    Parse::Post => {
                        post = Some(
                            part.parse::<Base36>()
                                .assumption("Id post number should be in base36")?,
                        );
                        state = Parse::Reply;
                    }
                    Parse::Reply => {
                        // This can only be reached if there is a reply part in
                        // the id, so we can wrap in `Some` unconditionally.
                        reply = Some(
                            part.parse::<Base36>()
                                .assumption("Id reply number should be in base36")?,
                        );
                    }
                }
            }

            debug_assert_matches!(
                state,
                Parse::Reply,
                "Id parsing should always end on a parsing reply state"
            );

            let id = Self {
                tag: tag.with_assumption(|| {
                    format!("`tag` in post Id should have been populated: `{str}`")
                })?,
                scope: scope.with_assumption(|| {
                    format!("`scope` in post Id should have been populated: `{str}`")
                })?,
                webtoon: webtoon.with_assumption(|| {
                    format!("`webtoon` in post Id should have been populated: `{str}`")
                })?,
                episode: episode.with_assumption(|| {
                    format!("`episode` in post Id should have been populated: `{str}`")
                })?,
                post: post.with_assumption(|| {
                    format!("`post` in post Id should have been populated: `{str}`")
                })?,
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
            // Cannot add `self.tag != other.tag` as its still unknown how this
            // number increments, but given that the other checks are enough to
            // know if the post is on the same Webtoon and the same episode it
            // should be fine.
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
        type Error = Assumption;

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
