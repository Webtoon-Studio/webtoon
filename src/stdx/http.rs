use anyhow::anyhow;
use reqwest::{RequestBuilder, Response};
use std::time::Duration;

pub static DEFAULT_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct Retry(RequestBuilder);

impl Retry {
    pub async fn send(self) -> Result<Response, anyhow::Error> {
        let mut tries = 10;
        let mut wait = fastrand::u64(1..=10);

        loop {
            let request = self
                .0
                .try_clone()
                .ok_or_else(|| anyhow!("failed to clone `RequestBuilder` in retry loop"))?;

            let response = request.send().await;

            match response {
                Ok(response) if response.status() == 429 && tries != 0 => {
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    tries -= 1;
                    wait += 3;
                    wait += fastrand::u64(1..=5);
                }
                Err(_) if tries != 0 => {
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    tries -= 1;
                    wait += 3;
                    wait += fastrand::u64(1..=5);
                }

                Ok(response) => return Ok(response),
                Err(err) => return Err(err.into()),
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
        const AGENT: &str = "webtoon/0.7.0";
        const { assert!(AGENT.len() == DEFAULT_USER_AGENT.len()) }
        assert_eq!(AGENT, DEFAULT_USER_AGENT);
    }
}
