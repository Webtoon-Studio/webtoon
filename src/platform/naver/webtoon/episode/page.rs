pub(super) mod panels;

use anyhow::Context;
use scraper::Html;

use self::panels::Panel;
use super::EpisodeError;

#[derive(Debug, Clone)]
pub struct Page {
    pub(super) note: Option<String>,
    pub(super) panels: Vec<Panel>,
}

impl Page {
    pub fn parse(str: &str, html: &Html, episode: u16) -> Result<Self, EpisodeError> {
        Ok(Self {
            note: note(str).context("Episode creator note failed to be parsed")?,
            panels: panels::from_html(html, episode)
                .context("Episode panel urls failed to be parsed")?,
        })
    }
}

fn note(str: &str) -> Result<Option<String>, EpisodeError> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Article {
        author_words: String,
    }

    for line in str.lines() {
        if !line.trim_start().starts_with("article: ") {
            continue;
        }

        let line = line
            .trim_start()
            .trim_start_matches("article: ")
            .trim_end_matches(',');

        let article: Article = serde_json::from_str(line) //
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        return Ok(Some(article.author_words));
    }

    Ok(None)
}
