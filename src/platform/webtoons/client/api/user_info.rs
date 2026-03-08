use serde::Deserialize;

use crate::stdx::error::{Assumption, assumption};

/// Represents data from the `webtoons.com/*/member/userInfo` endpoint.
///
/// This can be used to get the username and profile, as well as check if user is logged in. This type is not constructed
/// directly, but gotten through [`Client::user_info_for_session()`](crate::platform::webtoons::client::Client::user_info_for_session).
///
/// # Example
///
/// ```
/// # use webtoon::platform::webtoons::{error::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// if let Some(user_info) = client.user_info_for_session("session").await? {
///     assert!(user_info.is_canvas_creator());
///     assert_eq!("username", user_info.username());
///     assert_eq!(Some("profile"), user_info.profile());
///     # unreachable!("should be `None`");
/// }
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct UserInfo {
    is_canvas_creator: bool,
    username: String,
    profile: Option<String>,
}

impl UserInfo {
    /// Returns if current user is a canvas creator.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// if let Some(user) = client.user_info_for_session("session").await? {
    ///     assert!(user.is_canvas_creator());
    ///     # unreachable!("should be `None`");
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_canvas_creator(&self) -> bool {
        self.is_canvas_creator
    }

    /// Returns the users' username.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// if let Some(user_info) = client.user_info_for_session("session").await? {
    ///     assert_eq!("username", user_info.username());
    ///     # unreachable!("should be `None`");
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    /// Returns the profile segment for `webtoons.com/*/creator/{profile}`.
    ///
    /// If the session provided is invalid, then `profile` will be `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webtoon::platform::webtoons::{error::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// if let Some(user_info) = client.user_info_for_session("session").await? {
    ///     assert_eq!(Some("profile"), user_info.profile());
    ///     # unreachable!("should be `None`");
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }
}

impl TryFrom<UserInfoRaw> for UserInfo {
    type Error = Assumption;
    fn try_from(user: UserInfoRaw) -> Result<Self, Self::Error> {
        let Some(username) = user.username else {
            assumption!(
                "`UserInfoRaw::username` was `None`, and when using `try_from|into`, should be checked beforehand that it is `Some`"
            );
        };

        Ok(Self {
            is_canvas_creator: user.is_canvas_creator,
            username,
            profile: user.profile,
        })
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct UserInfoRaw {
    #[serde(rename = "challengeAuthor")]
    pub is_canvas_creator: bool,

    #[serde(rename = "loginUser")]
    pub is_logged_in: bool,

    #[serde(rename = "nickname")]
    pub username: Option<String>,

    #[serde(rename = "profileUrl")]
    pub profile: Option<String>,
}
