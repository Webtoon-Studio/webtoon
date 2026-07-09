use reqwest::{RequestBuilder, Response, StatusCode};
use std::time::Duration;

/// The default `User-Agent` header value, formatted as `{crate_name}/{crate_version}`.
pub static DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const TOO_MANY_REQUESTS: StatusCode = StatusCode::TOO_MANY_REQUESTS;
const INTERNAL_SERVER_ERROR: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

/// Wraps a [`RequestBuilder`] with automatic retry logic on failure or rate limiting.
pub struct Retry(RequestBuilder);

impl Retry {
    pub async fn send(self) -> Result<Response, reqwest::Error> {
        let mut tries: u32 = 10;
        let mut wait = fastrand::u64(1..=5);

        loop {
            let request = self
                .0
                .try_clone()
                .expect("`RequestBuilder` should only fail to clone when working with streams; we only do standard requests");

            let should_retry = match request.send().await {
                Err(_) if tries > 0 => true,
                Err(err) => return Err(err),
                Ok(response)
                    if matches!(response.status(), TOO_MANY_REQUESTS | INTERNAL_SERVER_ERROR)
                        && tries > 0 =>
                {
                    true
                }
                Ok(response) => return Ok(response),
            };

            if should_retry {
                tokio::time::sleep(Duration::from_secs(wait)).await;
                tries -= 1;
                wait += 3 + fastrand::u64(1..=5);
            }
        }
    }
}

/// Extension trait for adding retry behavior to [`RequestBuilder`].
pub trait RequestExt {
    /// Wraps this [`RequestBuilder`] in a [`Retry`] that retries on failure or `429` responses.
    fn retry(self) -> Retry;
}

impl RequestExt for RequestBuilder {
    fn retry(self) -> Retry {
        Retry(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_user_agent_should_be_expected() {
        const AGENT: &str = "webtoon/0.9.0";
        const { assert!(AGENT.len() == DEFAULT_USER_AGENT.len()) }
        assert_eq!(AGENT, DEFAULT_USER_AGENT);
    }
}
