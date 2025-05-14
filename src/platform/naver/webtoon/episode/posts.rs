//! Module containing things related to posts and their posters.

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use core::fmt;
use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::{
    platform::{
        naver::client::posts::CommentList,
        naver::{Webtoon, errors::PostError},
    },
    private::Sealed,
};

use super::Episode;

/// Represents a collection of posts.
///
/// Can be though of as a wrapper for a `Vec<Post>` to provide methods on to further interact.
///
/// This type is not constructed directly, but gotten via [`Webtoon::posts()`](Webtoon::posts()),
/// [`Episode::posts()`](Episode::posts()) or [`Post::replies()`](Post::replies()).
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
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     let posts = episode.posts().await?;
    ///
    ///     for post in posts.filter(|post| post.is_top()) {
    ///         println!("only the best: {:#post}");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Performs an inplace, unstable sort of the upvotes, from largest to smallest.
    pub fn sort_by_upvotes(&mut self) {
        self.posts
            .sort_unstable_by(|a, b| b.upvotes.cmp(&a.upvotes));
    }

    /// Return the underlying `Vec<Post>` as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[Post] {
        &self.posts
    }

    /// Returns the number of posts gotten.
    ///
    /// For just the total count, you should instead use [`Episode::comments_and_replies()`](Episode::comments_and_replies())
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     let posts = episode.posts().await?;
    ///
    ///     // For this `count` use case, you could instead use `Episode::comments_and_replies()`
    ///     println!("`{}` has `{}` total comments", webtoon.title(), posts.count());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn count(&self) -> usize {
        self.posts.len()
    }
}

/// Represents a post on `comic.naver.com`, either a reply or a top-level comment.
#[derive(Clone)]
pub struct Post {
    pub(crate) episode: Episode,
    pub(crate) id: String,
    pub(crate) parent_id: String,
    pub(crate) body: String,
    pub(crate) upvotes: u32,
    pub(crate) downvotes: u32,
    pub(crate) replies: u32,
    pub(crate) is_top: bool,
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
            .field("posted", &self.posted)
            .field("poster", &self.poster)
            .finish()
    }
}

impl Post {
    /// Returns the [`Poster`] of post.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for posts in episode.posts().await? {
    ///        println!("poster: {}", post.poster().username());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn poster(&self) -> &Poster {
        &self.poster
    }

    /// Returns the unique id for the post.
    ///
    /// The platform id is a positive integer stored as a string: `"12089402312"`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for posts in episode.posts().await? {
    ///        println!("post id: {}", post.id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the parent id of the post.
    ///
    /// If the post is a top-level comment, the parent `id` will be the same as the post's own [`id()`](Post::id()). If the post is
    /// a reply to another comment, the parent `id` will reflect the `id` of the post it is replying to.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post's parent id: {}", post.parent_id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn parent_id(&self) -> &str {
        &self.parent_id
    }

    /// Returns the actual content of the post.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post's body of text: {}", post.body());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns how many upvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post's upvotes: {}", post.upvotes());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    /// Returns how many downvotes the post has.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post's upvotes: {}", post.downvotes());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn downvotes(&self) -> u32 {
        self.downvotes
    }

    /// Returns whether this post is a top-level comment and not a reply.
    ///
    /// This is functionally equivalent to `Post::id() == Post::parent_id()`;
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post is a {}", if post.is_comment() { "comment" } else { "reply" } );
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_comment(&self) -> bool {
        self.id == self.parent_id
    }

    /// Returns whether this post is a reply and not a top-level comment.
    ///
    /// This is functionally equivalent to `Post::id() != Post::parent_id()`;
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         println!("post is a {}", if post.is_reply() { "reply" } else { "comment" } );
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.id != self.parent_id
    }

    /// Returns whether this post is a `TOP` post, one of the posts on the first page of the episode.
    ///
    /// If posts are gotten with [`Episode::posts_for_each()`] this will always be false.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(50).await? {
    ///     for post in episode.posts().await? {
    ///         if post.is_top() {
    ///             println!("{} left a top comment!", post.poster().username());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.is_top
    }

    /// Returns the episode number of the post was left on.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(25).await? {
    ///     for post in episode.posts().await? {
    ///         println!("{} left a comment on episode `{}`", post.poster().username(), post.episode());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in an `ISO 8601` millisecond timestamp format.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(25).await? {
    ///     for post in episode.posts().await? {
    ///         println!("posted `date:{}`", post.posted());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn posted(&self) -> i64 {
        self.posted.timestamp_millis()
    }

    /// Returns the replies on the current post.
    ///
    /// The return type depends on the specified output type and can either return the total number of replies or a collection of the actual replies.
    ///
    /// # Return Types
    ///
    /// - For `u32`: Returns the count of replies.
    /// - For `Posts`: Returns the replies as a [`Posts`] object, with replies sorted from to newest to oldest.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(813443).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(25).await? {
    ///     for post in episode.posts().await? {
    ///         let replies: u32 = post.replies().await?;
    ///         println!("`{}` has `{replies}` replies", post.id());
    ///
    ///         for reply in post.replies::<Posts>().await? {
    ///             println!("{} left a reply to {}", reply.poster().username(), post.poster().username());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn replies<R: Replies>(&self) -> Result<R, PostError> {
        R::replies(self).await
    }
}

impl TryFrom<(&Episode, CommentList)> for Post {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from((episode, post): (&Episode, CommentList)) -> Result<Self, Self::Error> {
        let id = if let Some(id) = post.id_no {
            id
        } else if let Some(id) = post.user_id_no {
            id
        } else if let Some(id) = post.profile_user_id {
            id
        } else {
            bail!("failed to find a user id for post that wasn't `null`")
        };

        Ok(Self {
            episode: episode.clone(),
            id: post.comment_no.clone(),
            parent_id: post.parent_comment_no,
            body: post.contents,
            upvotes: post.sympathy_count,
            downvotes: post.antipathy_count,
            replies: post.reply_count,
            is_top: post.best,
            posted: post
                .mod_time
                .parse::<DateTime<Utc>>()
                .with_context(|| format!("`{}` is not a valid timestamp", post.mod_time))?,
            poster: Poster {
                webtoon: episode.webtoon.clone(),
                episode: episode.number,
                post_id: post.comment_no,
                id,
                username: post.user_name,
                is_creator: post.manager,
            },
        })
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

/// Represents information about the poster of a [`Post`].
///
/// # Example
///
/// ```
/// # use webtoon::platform::naver::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let Some(webtoon) = client.webtoon(837999).await? else {
///     unreachable!("webtoon is known to exist");
/// };
///
/// if let Some(episode) = webtoon.episode(1) {
///     for post in episode.posts().await? {
///         let poster = post.poster();
///
///         println!("poster: {}", poster.username());
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[allow(unused)]
#[derive(Clone)]
pub struct Poster {
    webtoon: Webtoon,
    episode: u16,
    post_id: String,
    pub(crate) id: String,
    pub(crate) username: String,
    pub(crate) is_creator: bool,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Poster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Poster")
            // omitting `webtoon`
            .field("id", &self.id)
            .field("username", &self.username)
            .field("is_creator", &self.is_creator)
            .finish()
    }
}

impl Poster {
    /// Returns the posters id.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(837999).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
    ///     for post in episode.posts().await? {
    ///         let poster = post.poster();
    ///
    ///         println!("poster id: `{}`", poster.id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns poster username.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(837999).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
    ///     for post in episode.posts().await? {
    ///         let poster = post.poster();
    ///
    ///         println!("poster: {}", poster.username());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns if poster is a creator on the `comic.naver.com` platform.
    ///
    /// This doesn't mean they are the creator of the current Webtoon, just that they are a creator, though it could be of the current Webtoon.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::naver::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let Some(webtoon) = client.webtoon(837999).await? else {
    ///     unreachable!("webtoon is known to exist");
    /// };
    ///
    /// if let Some(episode) = webtoon.episode(1) {
    ///     for post in episode.posts().await? {
    ///         if post.poster().is_creator() {
    ///             println!("{} is a creator in the platform!", poster.username());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_creator(&self) -> bool {
        self.is_creator
    }
}

/// Trait providing a way to be generic over what type is returned for `replies`.
///
/// This was made so that [`Post`] can have a single [`replies()`](Post::replies()) method, but provide the ability
/// to get the posts themselves as well as the count without having to come up with another name.
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
        // No need to make a network request when there are no replies to fetch.
        if post.replies == 0 {
            return Ok(Self { posts: Vec::new() });
        }

        #[allow(clippy::mutable_key_type)]
        let mut replies = HashSet::new();

        let response = post
            .episode
            .webtoon
            .client
            .get_replies_for_post(post, 1)
            .await?
            .text()
            .await?;

        let text = response
            .trim_start_matches("_callback(")
            .trim_end_matches(");");

        let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
            .with_context(|| text.to_string())?;

        let pages = api.result.page_model.total_pages;

        // Add first posts
        for reply in api.result.comment_list {
            if reply.deleted {
                continue;
            }

            replies
                .insert(Post::try_from((&post.episode, reply)).with_context(|| text.to_string())?);
        }

        for page in 2..pages {
            let response = post
                .episode
                .webtoon
                .client
                .get_replies_for_post(post, page)
                .await?
                .text()
                .await?;

            let text = response
                .trim_start_matches("_callback(")
                .trim_end_matches(");");

            let api = serde_json::from_str::<crate::platform::naver::client::posts::Posts>(text)
                .with_context(|| text.to_string())?;

            for reply in api.result.comment_list {
                if reply.deleted {
                    continue;
                }

                replies.insert(
                    Post::try_from((&post.episode, reply)).with_context(|| text.to_string())?,
                );
            }
        }

        let replies = Self {
            posts: replies.into_iter().collect(),
        };

        Ok(replies)
    }
}
