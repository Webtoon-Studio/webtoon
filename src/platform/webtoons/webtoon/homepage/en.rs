use anyhow::Context;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use scraper::{ElementRef, Html, Selector};
use std::{str::FromStr, sync::Arc};
use url::Url;

use crate::{
    platform::webtoons::{
        Client, Language, Webtoon,
        creator::Creator,
        meta::{Genre, Scope},
        originals::Schedule,
        webtoon::{WebtoonError, episode::Episode},
    },
    stdx::math::MathExt,
};

use super::Page;

pub(super) fn page(html: &Html, webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let page = match webtoon.scope {
        Scope::Original(_) => Page {
            title: title(html)?,
            creators: creators(html, &webtoon.client)?,
            genres: genres(html)?,
            summary: summary(html)?,
            views: views(html)?,
            subscribers: subscribers(html)?,
            schedule: Some(schedule(html)?),
            thumbnail: None,
            banner: Some(banner(html)?),
            pages: calculate_total_pages(html)?,
        },
        Scope::Canvas => Page {
            title: title(html)?,
            creators: creators(html, &webtoon.client)?,
            genres: genres(html)?,
            summary: summary(html)?,
            views: views(html)?,
            subscribers: subscribers(html)?,
            schedule: None,
            thumbnail: Some(canvas_thumbnail(html)?),
            banner: Some(banner(html)?),
            pages: calculate_total_pages(html)?,
        },
    };

    Ok(page)
}

pub(super) fn title(html: &Html) -> Result<String, WebtoonError> {
    // `h1.subj` for featured `h3.subj` for challenge_list.
    let selector = Selector::parse(r".subj") //
        .expect("`.subj` should be a valid selector");

    // The first occurrence of the element is the desired story's title.
    // The other instances are from recommended stories and are not wanted here.
    let selected = html
        .select(&selector)
        .next()
        .context("`.subj` is missing: webtoons requires a title")?;

    let mut title = String::new();

    // element can have a `<br>` in the middle of the next, for example https://www.webtoons.com/en/romance/the-reason-for-the-twin-ladys-disguise/list?title_no=6315
    // so need to iterate over all isolated text blocks.
    for text in selected.text() {
        // Similar to an creator name that can have random whitespace around the actual title, so too can story titles. Nerd and Jock is one such example.
        for word in text.split_whitespace() {
            title.push_str(word);
            title.push(' ');
        }
    }

    // Removes the extra space added at the end in the prior loop
    title.pop();

    Ok(title)
}

pub(super) fn creators(html: &Html, client: &Client) -> Result<Vec<Creator>, WebtoonError> {
    // NOTE: Some creators have a little popup when you click on a button. Other have a dedicated page on the platform.
    // All instances have a `div.author_area` but the ones with a button have the name located directly in this.
    // Other instances have a nested <a> tag with the name.
    //
    // Platform creators, that is those that have a `webtoons.com` account, which is required to upload to the site,
    // have a profile on the site that follows a `/en/creator/PROFILE` pattern. However, as the website also uploads
    // translated versions of stories from Naver Comics, those creators do not have any `webtoons.com` account as
    // they don't need one to upload there. Naver Comics are an example, but this would be the case where there
    // doesn't need to be an account to upload a story.
    //
    // The issue this creates, however, is that these non-accounts have a different html and css structure.
    // And other issues, like commas, which separate multiple creators of a story, and abbreviations, like `...`
    // that indicate the end of a long list of creators also need to be filtered through.
    //
    // To make sure the problems don't end there, there can be a mic of account and non-account stories.
    // This case presents a true hell, but it doeable. But maintaining a working implementation here will
    // take a lot of effort.
    //
    // Currently only Originals can have multiple authors for a single webtoon.

    let selector = Selector::parse(r"a.author") //
        .expect("`a.author` should be a valid selector");

    let mut creators = Vec::new();

    // Canvas creator must have a `webtoons.com` account.
    for selected in html.select(&selector) {
        let url = selected
            .value()
            .attr("href")
            .context("`href` is missing, `a.author` should always have one")?;

        let url = Url::parse(url).map_err(|err| WebtoonError::Unexpected(err.into()))?;

        // creator profile should always be the last path segment
        // e.g. https://www.webtoons.com/en/creator/792o8
        let profile = url
            .path_segments()
            .context("`href` should have path segments")?
            .next_back()
            .unwrap();

        let mut username = String::new();

        for text in selected.text() {
            // This is for cases where Webtoon's, for some ungodly reason, is putting a bunch of tabs and new-lines in the name.
            // 66,666 Years: Advent of the Dark Mage is the first example. Archie Comics: Big Ethel Energy is another, as well as Tower of God.
            for text in text.split_whitespace() {
                username.push_str(text);
                username.push(' ');
            }

            // Remove trailing space
            username.pop();
        }

        creators.push(Creator {
            client: client.clone(),
            language: Language::En,
            profile: Some(profile.into()),
            username,
            page: Arc::new(RwLock::new(None)),
        });
    }

    let selector = Selector::parse(r"div.author_area") //
        .expect("`div.author_area` should be a valid selector");

    // Originals creators that have no Webtoon account, or a mix of no accounts and `webtoons.com` accounts.
    if let Some(selected) = html.select(&selector).next() {
        for text in selected.text() {
            // The last text block in the element meaning all creators have been gone through.
            if text == "author info" {
                break;
            }

            'username: for username in text.split(',') {
                let username = username.trim().trim_end_matches("...").trim();
                if username.is_empty() {
                    continue;
                }

                // `webtoons.com` creators have their name come up again in this loop.
                // The text should be the exact same so its safe to check if they already exist in the vector,
                // continuing to the next text block if so.
                for creator in &creators {
                    if creator.username == username {
                        continue 'username;
                    }
                }

                creators.push(Creator {
                    client: client.clone(),
                    language: Language::En,
                    profile: None,
                    username: username.trim().into(),
                    page: Arc::new(RwLock::new(None)),
                });
            }
        }
    }

    Ok(creators)
}

pub(super) fn genres(html: &Html) -> Result<Vec<Genre>, WebtoonError> {
    // `h2.genre` for originals and `p.genre` for canvas
    // Doing just `.genre` gets the all instances of the class
    let selector = Selector::parse(r".info>.genre") //
        .expect("`.info>.genre` should be a valid selector");

    let mut genres = Vec::with_capacity(2);

    for selected in html.select(&selector) {
        let genre = selected
            .text()
            .next()
            .context("`.info>.genre` was found but no text was present")?;

        genres.push(Genre::from_str(genre).map_err(|err| WebtoonError::Unexpected(err.into()))?);
    }

    if genres.is_empty() {
        return Err(WebtoonError::NoGenre);
    }

    Ok(genres)
}

pub(super) fn views(html: &Html) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let views = html
        .select(&selector)
        .next()
        .context("`em.cnt` is missing: webtoons page displays total views")?
        .inner_html();

    match views.as_str() {
        billion if billion.ends_with('B') => {
            let billion = billion
                .trim_end_matches('B')
                .parse::<f64>()
                .context(views)?;

            Ok((billion * 1_000_000_000.0) as u64)
        }
        million if million.ends_with('M') => {
            let million = million
                .trim_end_matches('M')
                .parse::<f64>()
                .context(views)?;

            Ok((million * 1_000_000.0) as u64)
        }
        thousand => Ok(thousand.replace(',', "").parse::<u64>().context(views)?),
    }
}

pub(super) fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let subscribers = html
        .select(&selector)
        .nth(1)
        .context("`em.cnt` is missing: webtoons page displays subscribers")?
        .inner_html();

    match subscribers.as_str() {
        million if million.ends_with('M') => {
            let million = million
                .trim_end_matches('M')
                .parse::<f64>()
                .context(subscribers)?;

            Ok((million * 1_000_000.0) as u32)
        }
        thousand => Ok(thousand
            .replace(',', "")
            .parse::<u32>()
            .context(subscribers)?),
    }
}

// NOTE: Could also parse from the json on the story page `logParam`
// *ONLY* for Originals.
pub(super) fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info").expect("`p.day_info` should be a valid selector");

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .context("`p.day_info` is missing: webtoons displays a release schedule")?
        .text()
    {
        if text == "UP" {
            continue;
        }

        for release in text.split_whitespace() {
            // `MON,` -> `MON`
            let release = release.trim_end_matches(',');

            if release == "EVERY" {
                continue;
            }

            releases.push(release);
        }
    }

    let schedule = Schedule::try_from(releases) //
        .map_err(|err| WebtoonError::Unexpected(err.into()))?;

    Ok(schedule)
}

pub(super) fn summary(html: &Html) -> Result<String, WebtoonError> {
    let selector = Selector::parse(r"p.summary") //
        .expect("`p.summary` should be a valid selector");

    let text = html
        .select(&selector)
        .next()
        .context("`p.summary` is missing: webtoons requires a summary")?
        .text()
        .next()
        .context("`p.summary` was found but no text was present")?;

    let mut summary = String::new();

    // Gets rid of any weird formatting, such as newlines and tabs being in the middle of the summary.
    for word in text.split_whitespace() {
        summary.push_str(word);
        summary.push(' ');
    }

    // Removes the final spacing at the end while keeping it a string.
    summary.pop();

    Ok(summary)
}

pub fn _original_thumbnail(_html: &Html) -> Result<Url, WebtoonError> {
    todo!()
}

pub(super) fn canvas_thumbnail(html: &Html) -> Result<Url, WebtoonError> {
    // `h1.subj` for featured `h3.subj` for challenge_list.
    let selector = Selector::parse(r".thmb>img") //
        .expect("`.thmb>img` should be a valid selector");

    let url = html
        .select(&selector)
        .next()
        .context(
            "`thmb>img` is missing: canvas webtons should have a thumnbail displayed on the page",
        )?
        .attr("src")
        .context("`src` is missing, `.thmb>img` should always have one")?;

    let mut thumbnail = Url::parse(url)?;

    thumbnail
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

    Ok(thumbnail)
}

pub(super) fn banner(html: &Html) -> Result<Url, WebtoonError> {
    // `h1.subj` for featured `h3.subj` for challenge_list.
    let selector = Selector::parse(r".thmb>img") //
        .expect("`.thmb>img` should be a valid selector");

    let url = html
        .select(&selector)
        .next()
        .context("`thmb>img` is missing: originals should display a banner image on the page")?
        .attr("src")
        .context("`src` is missing, `.thmb>img` should always have one")?;

    let mut banner = Url::parse(url)?;

    banner
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .expect("`swebtoon-phinf.pstatic.net` should be a valid host");

    Ok(banner)
}

pub fn calculate_total_pages(html: &Html) -> Result<u16, WebtoonError> {
    let selector = Selector::parse("li._episodeItem>a>span.tx") //
        .expect("`li._episodeItem>a>span.tx` should be a valid selector");

    // Counts the episodes listed per page. This is needed as there can be a varying amounts: 9 or 10, for example.
    let episodes_per_page = u16::try_from(html.select(&selector).count())
        .context("Episodes per page count wasnt able to fit within a u16")?;

    let selected = html.select(&selector).next().context(
        "`span.tx` was missing: webtoons page should have at least one episode if it is viewable",
    )?;

    let text = selected
        .text()
        .next()
        .context("`span.tx` was found but no text was present")?;

    if !text.starts_with('#') {
        return Err(WebtoonError::Unexpected(anyhow::anyhow!(
            "`{text}` is missing a `#` at the front of it"
        )));
    }

    let episode = text
        .trim_start_matches('#')
        .parse::<u16>()
        .map_err(|err| WebtoonError::Unexpected(err.into()))?;

    Ok(episode.in_bucket_of(episodes_per_page))
}

pub(super) fn episode(
    element: &ElementRef<'_>,
    webtoon: &Webtoon,
) -> Result<Episode, WebtoonError> {
    let title = episode_title(element)?;

    let number = element
        .value()
        .attr("data-episode-no")
        .context("attribute `data-episode-no` should be found on webtoon page with episodes on it")?
        .parse::<u16>()
        .context("`data-episode-no` should be an int")?;

    let published = episode_published_date(element)?;

    Ok(Episode {
        webtoon: webtoon.clone(),
        season: Arc::new(RwLock::new(super::super::episode::season(&title))),
        title: Arc::new(RwLock::new(Some(title))),
        number,
        published: Some(published),
        length: Arc::new(RwLock::new(None)),
        thumbnail: Arc::new(RwLock::new(None)),
        note: Arc::new(RwLock::new(None)),
        panels: Arc::new(RwLock::new(None)),
        views: None,
        // NOTE: Impossible to say from this page. In general any random Original episode would have been
        // behind fast-pass, but the initial release episodes which never were would be impossible to tell.
        // Same goes for Canvas. Impossible to say from just the info on this page.
        ad_status: None,
        published_status: Some(super::super::episode::PublishedStatus::Published),
    })
}

pub(super) fn episode_title(episode: &ElementRef<'_>) -> Result<String, WebtoonError> {
    let selector = Selector::parse("span.subj>span") //
        .expect("`span.subj>span` should be a valid selector");

    let title = episode
        .select(&selector)
        .next()
        .context("`span.subj>span` should exist on page with episodes")?
        .text()
        .next()
        .context("`span.subj>span` should have text inside it")?;

    let escaped = html_escape::decode_html_entities(title);

    Ok(escaped.to_string())
}

// NOTE: Currently forces all dates to be at 02:00 UTC as thats when the originals get released.
// For more accurate times, must have a session.
fn episode_published_date(episode: &ElementRef<'_>) -> Result<DateTime<Utc>, WebtoonError> {
    let selector = Selector::parse("span.date") //
        .expect("`span.date` should be a valid selector");

    let mut date = episode
        .select(&selector)
        .next()
        .context("`span.date` should be found on a webtoon page with episodes on it")?
        .text()
        .next()
        .context("`span.date` should have text inside it")?
        .trim()
        .to_owned();

    date.push_str(" 02:00:00 +0000");

    // %b %e, %Y -> Jun 3, 2022
    // %b %d, %Y -> Jun 03, 2022
    // %F -> 2022-06-03 (ISO 8601)

    let date = DateTime::parse_from_str(&date, "%b %e, %Y %T %z")
        .context("english webtoon page should have dates of pattern `Jun 3, 2022`")?;

    Ok(date.into())
}
