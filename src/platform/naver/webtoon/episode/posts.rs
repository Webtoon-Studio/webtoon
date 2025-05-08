//! Module containing things related to posts and their posters.

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use core::fmt;
use std::{cmp::Ordering, collections::HashSet, hash::Hash};

pub use crate::platform::naver::client::posts::Sort;

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
    #[must_use]
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
    #[must_use]
    pub fn poster(&self) -> &Poster {
        &self.poster
    }

    /// Returns the unique id for the post.
    ///
    /// The returned id contains all the necessary information to uniquely identify the post
    /// in the context of a specific Webtoon episode. This includes the Webtoon ID,
    /// episode number, post identifier, and optionally a reply identifier if the post is a reply.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::posts::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// # let episode = webtoon.episode(1).await?.expect("episode one shoudl exist");
    /// for post in  episode.posts(Sort::New).await? {
    ///     println!("Post ID: {:?}", post.id());
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the parent id of the post.
    ///
    /// If the post is a top-level comment, the parent ID will be the same as the post's own [`Self::id`].
    /// If the post is a reply to another comment, the parent ID will reflect the ID of the post it is replying to.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::posts::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// # let episode = webtoon.episode(1).await?.expect("episode one shoudl exist");
    /// # let posts = episode.posts(Sort::Best).await?;
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
    #[must_use]
    pub fn parent_id(&self) -> &str {
        &self.parent_id
    }

    /// Returns the actual content of the post.
    /// ### Example
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::posts::Sort};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// # let episode = webtoon.episode(1).await?.expect("episode one shoudl exist");
    /// # let posts = episode.posts(Sort::Best).await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// println!("Post content: {}", post.body());
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn body(&self) -> &str {
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

    /// Returns whether this post is a `TOP` post, one of the posts on the first page of the episode.
    ///
    /// If posts are gotten with `Sort::New` this will always be false.
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.is_top
    }

    /// Returns the episode number of the post was left on.
    #[must_use]
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in an `ISO 8601` millisecond timestamp format.
    #[must_use]
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
    /// # Usage
    ///
    /// Depending on the type you specify, you can either retrieve the number of replies or the actual replies themselves:
    ///
    /// ```rust
    /// # use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::posts::{Posts, Sort}};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(838432).await? {
    /// # let episode = webtoon.episode(1).await?.expect("episode 1 should exist");
    /// # let posts = episode.posts(Sort::New).await?;
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
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns poster username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns if poster is a creator on the `comic.naver.com` platform.
    ///
    /// This doesn't mean they are the creator of the current Webtoon, just that they are a creator, though it could be of the current Webtoon.
    pub fn is_creator(&self) -> bool {
        self.is_creator
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
