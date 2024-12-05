pub(super) mod panels;

use std::io;

use anyhow::Context;
use scraper::{Html, Selector};
use url::Url;

use self::panels::Panel;

use super::EpisodeError;

#[derive(Debug, Clone)]
pub struct Page {
    pub(super) title: String,
    pub(super) thumbnail: Url,
    pub(super) note: Option<String>,
    pub(super) panels: Vec<Panel>,
}

impl Page {
    pub fn parse(html: &Html, episode: u16) -> Result<Self, EpisodeError> {
        Ok(Self {
            title: title(html).context("Episode title failed to be parsed")?,
            thumbnail: thumbnail(html, episode).context("Episode thumbnail failed to be parsed")?,
            note: None,
            panels: panels::from_html(html, episode)
                .context("Episode panel urls failed to be parsed")?,
        })
    }
}

fn title(html: &Html) -> Result<String, EpisodeError> {
    let selector = Selector::parse("span#subTitle_toolbar") //
        .expect("`span#subTitle_toolbar` should be a valid selector");

    let title = html
            .select(&selector)
            .next()
            .context("`span#subTitle_toolbar` is missing: episode page should always contain a title for the episode")?
            .text()
            .next()
            .context("`span#subTitle_toolbar` was found but no text was present")?;

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn note(_html: &Html) -> Result<Option<String>, EpisodeError> {
    todo!()
}

fn thumbnail(html: &Html, episode: u16) -> Result<Url, EpisodeError> {
    let selector = Selector::parse(r"div.flicking-viewport>div.flicking-camera>div>a>img") //
        .expect(
            r"`div.flicking-viewport>div.flicking-camera>div>a>img` should be a valid selector",
        );

    for img in html.select(&selector) {
        let src = img
            .attr("src")
            .context("`src` is missing, `img` should always have one")?;

        let url = Url::parse(src).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        let number = url
            .path_segments()
            .context("episode thumbnail should always have segments")?
            .nth(2)
            .context("episode thumnbail url should have at least three segments")?
            .parse::<u16>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        if number != episode {
            continue;
        }

        return Ok(url);
    }

    Err(EpisodeError::NoThumbnailFound)
}
