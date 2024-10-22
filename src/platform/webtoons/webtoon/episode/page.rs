pub(super) mod panel;

use anyhow::Context;
use scraper::{Html, Selector};
use url::Url;

use self::panel::{Panel, Panels};
use super::EpisodeError;

#[derive(Debug, Clone)]
pub struct Page {
    pub(super) title: String,
    pub(super) thumbnail: Url,
    pub(super) length: u32,
    pub(super) note: Option<String>,
    pub(super) panels: Vec<Panel>,
}

impl Page {
    pub fn parse(html: &Html, episode: u16) -> Result<Self, EpisodeError> {
        Ok(Self {
            title: title(html).context("Episode title failed to be parsed")?,
            thumbnail: thumbnail(html, episode).context("Episode thumbnail failed to be parsed")?,
            length: length(html).context("Episode length failed to be parsed")?,
            note: note(html).context("Episode creator note failed to be parsed")?,
            panels: Panels::from_html(html, episode)
                .context("Episode panel urls failed to be parsed")?,
        })
    }
}

fn title(html: &Html) -> Result<String, EpisodeError> {
    let selector = Selector::parse("div.subj_info>h1.subj_episode") //
        .expect("`div.subj_info>h1.subj_episode` should be a valid selector");

    let title = html
            .select(&selector)
            .next()
            .context("`h1.subj_episode` is missing: episode page should always contain a title for the episode")?
            .text()
            .next()
            .context("`h1.subj_episode` was found but no text was present")?;

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn length(html: &Html) -> Result<u32, EpisodeError> {
    let selector = Selector::parse(r"img._images") //
        .expect("`img._images` should be a valid selector");

    let mut length = 0;

    for img in html.select(&selector) {
        length += img
            .value()
            .attr("height")
            .context("`height` is missing, `img._images` should always have one")?
            .split('.')
            .next()
            .context("`height` attribute should be a float")?
            .parse::<u32>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;
    }

    if length == 0 {
        return Err(EpisodeError::NoPanelsFound);
    }

    Ok(length)
}

fn note(html: &Html) -> Result<Option<String>, EpisodeError> {
    let selector = Selector::parse(r".creator_note>.author_text") //
        .expect("`.creator_note>.author_text` should be a valid selector");

    let Some(selection) = html.select(&selector).next() else {
        return Ok(None);
    };

    let note = selection
        .text()
        .next()
        .context("`.author_text` was found but no text was present")?
        .to_owned();

    Ok(Some(note))
}

fn thumbnail(html: &Html, episode: u16) -> Result<Url, EpisodeError> {
    let selector =
        Selector::parse(r"div.episode_lst>div.episode_cont>ul>li") //
            .expect(r"`div.episode_lst>div.episode_cont>ul>li` should be a valid selector");

    for li in html.select(&selector) {
        let data_episode_no = li
            .attr("data-episode-no")
            .context("`data-episode-no` is missing, `li` should always have one")?
            .parse::<u16>()
            .map_err(|err| EpisodeError::Unexpected(err.into()))?;

        if data_episode_no != episode {
            continue;
        }

        let img_selection = Selector::parse("a>span.thmb>img._thumbnailImages")
            .expect("`a>span.thmb>img._thumbnailImages` should be a valid selector");

        let mut img = li.select(&img_selection);

        let url = img
            .next()
            .context(
                "`img._thumbnailImages` is missing: episode page page should have at least one",
            )?
            .attr("data-url")
            .context("`data-url` is missing, `img._thumbnailimages` should always have one")?;

        let mut thumbnail = Url::parse(url).map_err(|err| EpisodeError::Unexpected(err.into()))?;

        thumbnail
            // This host doesn't need a `referer` header to see the image.
            .set_host(Some("swebtoon-phinf.pstatic.net"))
            .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

        return Ok(thumbnail);
    }

    Err(EpisodeError::NoThumbnailFound)
}
