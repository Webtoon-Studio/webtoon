//! Module containing things related to posts and their posters.

use chrono::{DateTime, Utc};
use core::fmt;
use std::{cmp::Ordering, hash::Hash};

use crate::{
    platform::naver::{
        self,
        errors::{ClientError, PostError, PosterError, ReplyError},
        Webtoon,
    },
    private::Sealed,
};

use super::Episode;

pub(super) async fn posts() -> Result<Posts, ClientError> {
    todo!()
}

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

/// Represensts a post on `comic.naver.com`, either a reply or a top-level comment.
#[derive(Clone)]
pub struct Post {
    pub(crate) episode: Episode,
    pub(crate) id: u64,
    pub(crate) parent_id: u64,
    pub(crate) body: String,
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
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// let post_id = post.id();
    /// println!("Post ID: {:?}", post_id);
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The [`Id`] is a composite structure that reflects the internal format used by Webtoon's system.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the parent [`Id`] of the post.
    ///
    /// If the post is a top-level comment, the parent ID will be the same as the post's own [`Self::id`].
    /// If the post is a reply to another comment, the parent ID will reflect the ID of the post it is replying to.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
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
    pub fn parent_id(&self) -> u64 {
        self.parent_id
    }

    /// Returns a reference to the [`Body`] of the post.
    ///
    /// This method provides access to the content of the post and whether it contains spoilers.
    /// The body contains the actual text of the post along with a flag indicating if it is marked as a spoiler.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
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
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns how many upvotes the post has.
    pub fn upvotes(&self) -> u32 {
        self.upvotes
    }

    /// Returns how many downvotes the post has.
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
    pub fn episode(&self) -> u16 {
        self.episode.number()
    }

    /// Returns the posts' published date in an ISO 8601 millisecond timestamp format.
    pub fn posted(&self) -> i64 {
        self.posted.timestamp_millis()
    }

    /// Upvotes post via users session.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    async fn upvote(&self) -> Result<(u32, u32), PostError> {
        todo!()
    }

    /// Downvotes post via users session.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    async fn downvote(&self) -> Result<(u32, u32), PostError> {
        todo!()
    }

    /// Will clear any upvote or downvote the user might have on the post.
    ///
    /// # Returns
    ///
    /// Returns the updated values for upvotes and downvotes: `(upvotes, downvotes)`.
    async fn unvote(&self) -> Result<(u32, u32), PostError> {
        todo!()
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
    async fn upvotes_and_downvotes(&self) -> Result<(u32, u32), PostError> {
        todo!()
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
    /// ```rust,no_run
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
    /// ```rust,no_run
    /// # use webtoon::platform::webtoons::{ Client, Language, Type, errors::Error};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// # let client = Client::new();
    /// # if let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? {
    /// # let posts = webtoon.posts().await?;
    /// # if let Some(post) = posts.into_iter().next() {
    /// post.reply("I know right!", false).await?;
    /// post.reply("In the novel *spoiler*", true).await?;
    /// # }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors:
    /// - Returns a [`ReplyError`] if there is an issue during the post request.
    async fn reply(&self, body: &str, is_spoiler: bool) -> Result<(), ReplyError> {
        todo!()
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
    async fn delete(&self) -> Result<(), PostError> {
        todo!()
    }
}

impl TryFrom<(&Episode, naver::client::posts::CommentList)> for Post {
    type Error = anyhow::Error;

    fn try_from(
        (episode, post): (&Episode, naver::client::posts::CommentList),
    ) -> Result<Self, Self::Error> {
        let id = post
            .comment_no
            .parse()
            .expect("post `commentNo` should always be parsable to a u64");
        let parent_id = post
            .parent_comment_no
            .parse()
            .expect("post `parentCommentNo` should always be parsable to a u64");

        let posted = post
            .reg_time_gmt
            .parse()
            .expect("timestamp should always be parsable to a `DateTime<Utc>`");

        let poster_id = match (post.id_no, post.user_id_no, post.profile_user_id) {
            (Some(id), _, _) | (_, Some(id), _) | (_, _, Some(id)) => id,
            // NOTE: When a post is deleted the profile ids change to `null`. An empty string is being used as the default
            // until a better option presents itself.
            _ => String::new(),
        };

        Ok(Self {
            episode: episode.clone(),
            id,
            parent_id,
            body: post.contents,
            upvotes: post.upvotes,
            downvotes: post.downvotes,
            replies: post.reply_all_count,
            is_top: post.best,
            is_deleted: post.deleted,
            posted,
            poster: Poster {
                webtoon: episode.webtoon.clone(),
                episode: episode.number(),
                post_id: id,
                id: poster_id,
                username: post.username,
            },
        })
    }
}

/// Represents information about the poster of a [`Post`].
#[derive(Clone)]
pub struct Poster {
    webtoon: Webtoon,
    episode: u16,
    post_id: u64,
    pub(crate) id: String,
    pub(crate) username: String,
}

#[expect(clippy::missing_fields_in_debug)]
impl fmt::Debug for Poster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Poster")
            // omitting `webtoon`
            .field("episode", &self.episode)
            .field("id", &self.id)
            .field("username", &self.username)
            .finish()
    }
}

impl Poster {
    /// Returns the posters `CUID`.
    ///
    /// Not to be confused with a `UUID`: [cuid2](https://github.com/paralleldrive/cuid2).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns poster username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns if the session user reacted to post.
    ///
    /// Returns `true` if the user reacted, `false` if not.
    async fn reacted(&self) -> bool {
        todo!()
    }

    /// Returns if current session user is creator of post.
    ///
    /// If there is no session provided, this is always `false`.
    fn is_current_session_user(&self) -> bool {
        todo!()
    }

    /// Returns if poster is a creator on the `comic.naver` platform.
    ///
    /// This doesn't mean they are the creator of the current webtoon, just that they are a creator, though it could be of the current webtoon.
    /// For that info use [`Poster::is_current_webtoon_creator`].
    fn is_creator(&self) -> bool {
        todo!()
    }

    /// Returns if the session user is the creator of the current webtoon.
    fn is_current_webtoon_creator(&self) -> bool {
        todo!()
    }

    /// Will block poster for current webtoon.
    ///
    /// Session user must be creator of the webtoon to moderate it. If this is not the case
    /// [`PosterError::InvalidPermissions`] will be returned.
    ///
    /// If attempting to block self, [`PosterError::BlockSelf`] will be returned.
    async fn block(&self) -> Result<(), PosterError> {
        todo!()
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
    // let response = episode
    //     .webtoon
    //     .client
    //     .posts_for_episode_at_page(episode, 1, 1)
    //     .await?;

    // if response.status() == 404 {
    //     Ok(false)
    // } else {
    //     Ok(true)
    // }
    todo!()
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
// impl Replies for Posts {
//     async fn replies(post: &Post) -> Result<Self, PostError> {
//         // No need to make a network request when there are no replies to fetch.
//         if post.replies == 0 {
//             return Ok(Posts { posts: Vec::new() });
//         }
//         todo!()
//     }
// }
