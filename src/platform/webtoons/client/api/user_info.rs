use serde::Deserialize;

/// Represents data from the `webtoons.com/*/member/userInfo` endpoint.
///
/// This can be used to get the username and profile, as well as check if user is logged in. This type is not constructed
/// directly, but gotten through [`Client::user_info_for_session()`].
///
/// # Example
///
/// ```no_run
/// # use webtoon::platform::webtoons::{errors::Error, Client};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// let client = Client::new();
///
/// let user_info = client.user_info_for_session("session").await?;
///
/// assert!(!user_info.is_logged_in());
/// assert_eq!(Some("username"), user_info.username());
/// assert_eq!(Some("profile"), user_info.profile());
/// # Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug)]
pub struct UserInfo {
    #[serde(rename = "loginUser")]
    is_logged_in: bool,

    #[serde(rename = "nickname")]
    username: Option<String>,

    #[serde(rename = "profileUrl")]
    profile: Option<String>,
}

impl UserInfo {
    /// Returns if current user session is logged in.
    ///
    /// Functionally, this tells whether a session is valid or not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert!(!user_info.is_logged_in());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// Returns the users' username.
    ///
    /// If the session provided is invalid, then `username` will be `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert_eq!(Some("username"), user_info.username());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns the profile segment for `webtoons.com/*/creator/{profile}`.
    ///
    /// If the session provided is invalid, then `profile` will be `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use webtoon::platform::webtoons::{errors::Error, Client};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    /// let client = Client::new();
    ///
    /// let user_info = client.user_info_for_session("session").await?;
    ///
    /// assert_eq!(Some("profile"), user_info.profile());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }
}
