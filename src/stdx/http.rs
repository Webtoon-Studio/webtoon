use reqwest::{RequestBuilder, Response};
use std::time::Duration;

pub static DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct Retry(RequestBuilder);

impl Retry {
    pub async fn send(self) -> Result<Response, reqwest::Error> {
        let mut tries = 10;
        let mut wait = fastrand::u64(1..=5);

        loop {
            #[allow(clippy::expect_used, reason = "if `RequestBuilder` fails to clone, it means we are working on streams, which is not the assumption of operation!")]
            let request = self.0.try_clone()
                .expect("`RequestBuilder` should only fail to clone when working with streams/readers, and we only do standard requests");

            match request.send().await {
                Ok(response) if response.status() == 429 && tries > 0 => {
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    tries -= 1;
                    wait += 3;
                    wait += fastrand::u64(1..=5);
                }
                Err(_) if tries > 0 => {
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    tries -= 1;
                    wait += 3;
                    wait += fastrand::u64(1..=5);
                }
                Ok(response) => return Ok(response),
                Err(err) => return Err(err),
            }
        }
    }
}

pub trait IRetry {
    fn retry(self) -> Retry;
}

impl IRetry for RequestBuilder {
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
