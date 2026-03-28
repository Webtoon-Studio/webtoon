mod de;
mod en;
mod es;
mod fr;
mod id;
mod th;
// mod zh;

use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use std::{str::FromStr, time::Duration};
use url::Url;

use crate::{
    platform::webtoons::{
        Client, Webtoon,
        creator::Creator,
        meta::{Genre, Language, Scope},
        originals::Schedule,
        webtoon::episode::{Published, PublishedStatus},
    },
    stdx::{
        cache::Cache,
        error::{Assume, AssumeFor, Assumption, assumption},
        math::MathExt,
    },
};

use super::{WebtoonError, episode::Episode};

#[inline]
pub async fn scrape(webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let html = webtoon.client.webtoon_page(webtoon, None).await?;
    let page = Page::parse(&html, webtoon)?;
    Ok(page)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Page {
    title: String,
    creators: Vec<Creator>,
    genres: Vec<Genre>,
    summary: String,
    views: u64,
    subscribers: u32,
    schedule: Option<Schedule>,
    thumbnail: Option<Url>,
    banner: Option<Url>,
    pages: u16,
}

impl Page {
    #[inline]
    fn parse(html: &Html, webtoon: &Webtoon) -> Result<Self, WebtoonError> {
        let page = match webtoon.scope {
            Scope::Original(_) => Self {
                title: title(html)?,
                creators: creators(html, &webtoon.client, webtoon)?,
                genres: genres(html)?,
                summary: summary(html)?,
                views: views(html, webtoon)?,
                subscribers: subscribers(html, webtoon)?,
                schedule: Some(schedule(html, webtoon)?),
                thumbnail: None,
                banner: Some(banner(html)?),
                pages: calculate_total_pages(html)?,
            },
            Scope::Canvas => Self {
                title: title(html)?,
                creators: creators(html, &webtoon.client, webtoon)?,
                genres: genres(html)?,
                summary: summary(html)?,
                views: views(html, webtoon)?,
                subscribers: subscribers(html, webtoon)?,
                schedule: None,
                thumbnail: Some(thumbnail(html)?),
                banner: None,
                pages: calculate_total_pages(html)?,
            },
        };

        Ok(page)
    }

    #[inline]
    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub(crate) fn creators(&self) -> &[Creator] {
        &self.creators
    }

    #[inline]
    pub(crate) fn genres(&self) -> &[Genre] {
        &self.genres
    }

    #[inline]
    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    #[inline]
    pub(crate) fn views(&self) -> u64 {
        self.views
    }

    #[inline]
    pub(crate) fn subscribers(&self) -> u32 {
        self.subscribers
    }

    #[inline]
    pub(crate) fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    #[inline]
    pub(crate) fn thumbnail(&self) -> Option<&str> {
        self.thumbnail.as_ref().map(|url| url.as_str())
    }

    #[inline]
    pub(crate) fn banner(&self) -> Option<&str> {
        self.banner.as_ref().map(Url::as_str)
    }
}

fn title(html: &Html) -> Result<String, WebtoonError> {
    // `h1.subj` for originals, `h3.subj` for canvas.
    let selector = Selector::parse(r".subj") //
        .assumption("`.subj` should be a valid selector")?;

    // The first occurrence of the element is the desired story's title.
    // The other instances are from recommended stories and are not wanted here.
    let selected = html
        .select(&selector) //
        .next()
        .assumption("`.subj`(title) is missing on `webtoons.com` Webtoon homepage")?;

    let mut title = String::new();

    // Element can have a `<br>` in the middle of the text:
    //  - https://www.webtoons.com/en/romance/the-reason-for-the-twin-ladys-disguise/list?title_no=6315
    //
    //    <h1 class="subj">
    //     The Reason for the
    //     <br>
    //     Twin Lady’s Disguise
    //    </h1>
    //
    // Need to iterate all isolated text blocks.
    for text in selected.text() {
        // Similar to a creator name that can have random whitespace around the
        // actual title, so too can story titles. Nerd and Jock is one such example.
        //
        // We must build of the title in parts to handle such case.
        for word in text.split_whitespace() {
            title.push_str(word);
            title.push(' ');
        }
    }

    // Removes the extra space added at the end in the prior loop
    title.pop();

    Ok(title)
}

fn creators(html: &Html, client: &Client, webtoon: &Webtoon) -> Result<Vec<Creator>, WebtoonError> {
    // NOTE:
    // Some creators have a little popup when you click on a button. Others have
    // a dedicated page on the platform. All instances have a `div.author_area`
    // but the ones with a button have the name located directly in this. Other
    // instances have a nested `<a>` tag with the name.
    //
    // Platform creators, that is, those that have a `webtoons.com` account, which
    // is required to upload to the site, have a profile on the site that follows
    // a `/en/creator/PROFILE` pattern.
    //
    // However, as the platform also uploads translated versions of stories from
    // Naver Comics, and those creators do not have any `webtoons.com` account
    // as they don't upload personally. Naver Comics are an example, but this
    // would be the case for other corporation, like Marvel, upload on behalf of
    // the actual creators.
    //
    // The issue this creates, however, is that these non-accounts have a different
    // HTML and CSS structure. And other issues, like commas, which separate multiple
    // creators of a story, and abbreviations, like `...` that indicate the end
    // of a long list of creators also need to be filtered through.
    //
    // To make sure the problems don't end there, there can be a mix of account
    // and non-account stories. This case presents a true hell, but its doable.
    // Just will take a lot of effort.
    //
    // Currently only Originals can have multiple authors for a single Webtoon.

    let mut creators = Vec::new();

    // Canvas
    // NOTE: Not all language versions have this in their Canvas page. They instead
    // have the same as the Originals below. This is because not every language
    // has creator profiles.
    let selector = Selector::parse(r"a.author") //
        .assumption("`a.author` should be a valid selector")?;

    // Canvas creator must have a `webtoons.com` account.
    for selected in html.select(&selector) {
        let url = selected
                .value()
                .attr("href")
                .assumption("`href` is missing, `a.author` on a `webtoons.com` Canvas Webtoon homepage should always have one")?;

        let url = Url::parse(url) //
            .assumption("`webtoons.com` canvas Webtoon homepage should always return valid and wellformed urls")?;

        // Creator profile should always be the last path segment:
        //     - https://www.webtoons.com/en/creator/792o8
        let profile = url
                .path_segments()
                .assumption("`href` attribute on `webtoons.com` canvas Webtoon homepage should have path segments")?
                .next_back()
                .assumption("Creator homepage url on `webtoons.com` Canvas homepage had segments, but `next_back` failed")?;

        // This is for cases where `webtoons.com`, for some ungodly reason, is
        // surrounding the name with a bunch of tabs and newlines, even if only
        // one creator.
        //
        // Examples:
        // - 66,666 Years: Advent of the Dark Mage
        // - Archie Comics: Big Ethel Energy
        // - Tower of God
        // - Press Play, Sami
        let username = selected
            .text()
            .next()
            .map(|this| this.trim())
            .assumption("`webtoons.com` creator text element should always be populated")?;

        let creator = Creator {
            client: client.clone(),
            language: webtoon.language(),
            username: username.to_string(),
            profile: Some(profile.into()),
            homepage: Cache::empty(),
        };

        creators.push(creator);
    }

    // Originals
    let selector = Selector::parse(r"div.author_area") //
        .assumption("`div.author_area` should be a valid selector")?;

    // Originals creators that have no Webtoon account, or a mix of no accounts and `webtoons.com` accounts.
    if let Some(selected) = html.select(&selector).next() {
        for username in usernames(selected) {
            // If any Creators were encountered again, ignore, as the `Creator`
            // added to the `Vec` in the Canvas section holds more info, like
            // having `Some` for `profile`.
            if !creators
                .iter()
                .any(|creator| creator.username() == username)
            {
                creators.push(Creator {
                    client: client.clone(),
                    profile: None,
                    username,
                    language: webtoon.language(),
                    homepage: Cache::empty(),
                });
            }
        }
    }

    if webtoon.is_canvas() {
        assumption!(
            creators.len() == 1,
            "`webtoons.com` canvas Webtoon homepages should have exactly one creator account associated with the Webtoon, got: {creators:?}"
        );
        return Ok(creators);
    }

    assumption!(
        !creators.is_empty(),
        "`webtoons.com` Webtoons must have some creator associated with them and displayed on their homepages"
    );

    Ok(creators)
}

fn genres(html: &Html) -> Result<Vec<Genre>, WebtoonError> {
    // `h2.genre` for originals and `p.genre` for canvas.
    //
    // Doing just `.genre` gets the all instances of the class
    let selector = Selector::parse(r".info>.genre") //
        .assumption("`.info>.genre` should be a valid selector")?;

    let mut genres = Vec::with_capacity(2);

    for selected in html.select(&selector) {
        let text = selected
            .text()
            .next()
            .assumption(" the `.info>.genre` tag was found on `webtoons.com` Webtoon homepage, but no text was present inside the element")?;

        let genre = match Genre::from_str(text) {
            Ok(genre) => genre,
            Err(err) => {
                assumption!("`webtoons.com` Webtoon homepage had an unexpected genre: {err}")
            }
        };

        genres.push(genre);
    }

    match genres.as_slice() {
        [_] | [_, _] => Ok(genres),
        [] => assumption!("no genre was found on `webtoons.com` Webtoon homepage"),
        [_, _, _, ..] => {
            assumption!("more than two genres were found on `webtoons.com` Webtoon homepage")
        }
    }
}

fn views(html: &Html, webtoon: &Webtoon) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .assumption("`em.cnt` should be a valid selector")?;

    let views = html
        .select(&selector)
        // First occurrence of `em.cnt` is for the views.
        .next()
        .assumption("`em.cnt`(views) element is missing on `webtoons.com` Webtoon homepage")?
        .inner_html();

    assumption!(
        !views.is_empty(),
        "views element(`em.cnt`) on `webtoons.com` Webtoon homepage should never be empty"
    );

    let views = match webtoon.language() {
        Language::En => en::views(&views)?,
        Language::Zh => todo!(),
        Language::Th => th::views(&views)?,
        Language::Id => id::views(&views)?,
        Language::Es => es::views(&views)?,
        Language::Fr => fr::views(&views)?,
        Language::De => de::views(&views)?,
    };

    Ok(views)
}

fn subscribers(html: &Html, webtoon: &Webtoon) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .assumption("`em.cnt` should be a valid selector")?;

    let subscribers = html
        .select(&selector)
        // First instance of `em.cnt` is for views.
        .nth(1)
        .assumption("second instance of `em.cnt`(subscribers) on `webtoons.com` Webtoon homepage is missing")?
        .inner_html();

    assumption!(
        !subscribers.is_empty(),
        "subscriber element(`em.cnt`) on `webtoons.com` Webtoon homepage should never be empty"
    );

    let subscribers = match webtoon.language() {
        Language::En => en::subscribers(&subscribers)?,
        Language::Zh => todo!(),
        Language::Th => th::subscribers(&subscribers)?,
        Language::Id => id::subscribers(&subscribers)?,
        Language::Es => es::subscribers(&subscribers)?,
        Language::Fr => fr::subscribers(&subscribers)?,
        Language::De => de::subscribers(&subscribers)?,
    };

    Ok(subscribers as u32)
}

fn schedule(html: &Html, webtoon: &Webtoon) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info") //
        .assumption("`p.day_info` should be a valid selector")?;

    let mut releases = Vec::new();

    for status in html
        .select(&selector)
        .next()
        .assumption("`p.day_info`(schedule) on `webtoons.com` originals Webtoons is missing")?
        .children()
        // Skip the <span> (icons like "UP")
        .filter_map(|node| node.value().as_text())
        .map(|text| text.trim())
        .find(|text| !text.is_empty())
        // Language specific cleaning so that only status or day/s are remaining.
        .map(|text| match webtoon.language() {
            Language::En => en::schedule(text),
            Language::Zh => todo!(),
            Language::Th => th::schedule(text),
            Language::Id => id::schedule(text),
            Language::Es => es::schedule(text),
            Language::Fr => fr::schedule(text),
            Language::De => de::schedule(text),
        })
        .assumption("`p.day_info`(schedule) should produce some form of non-empty text")?
        .split_whitespace()
        // `MON,` -> `MON`
        .map(|text| text.trim_end_matches(','))
    {
        releases.push(status);
    }

    assumption!(
        !releases.is_empty(),
        "original Webtoon homepage on `webtoons.com` should always have a release schedule, even if completed"
    );

    assumption!(
        releases.len() < 7,
        "original Webtoon homepage on `webtoons.com` should always have 6 or less items, as if there was a list of 7 days, it would just say `daily` instead: `{releases:?}`"
    );

    let schedule = match Schedule::try_from(releases) {
        Ok(schedule) => schedule,
        Err(err) => assumption!(
            "originals on `webtoons.com` should only have a few known release days/types: {err}"
        ),
    };

    Ok(schedule)
}

fn summary(html: &Html) -> Result<String, WebtoonError> {
    let selector = Selector::parse(r"p.summary") //
        .assumption("`p.summary` should be a valid selector")?;

    let text = html
        .select(&selector)
        .next()
        .assumption("`p.summary`(summary) on `webtoons.com` Webtoon homepage is missing")?
        .text()
        .next()
        .assumption(
            "`p.summary` on `webtoons.com` Webtoon homepage was found, but no text was present",
        )?;

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

// NOTE: originals had their homepage thumbnails removed, so only canvas has one we can get.
fn thumbnail(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .assumption("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .assumption("`thmb>img`(thumbail) on `webtoons.com` canvas Webtoon homepage is missing")?
        .attr("src")
        .assumption(
            "`src` attribute is missing in `.thmb>img` on `webtoons.com` canvas Webtoon homepage",
        )?;

    let mut thumbnail = match Url::parse(url) {
        Ok(thumbnail) => thumbnail,
        Err(err) => assumption!(
            "thumnbail url returned from `webtoons.com` canvas Webtoon homepage was an invalid absolute path url: {err}\n\n{url}"
        ),
    };

    thumbnail
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .assumption("`swebtoon-phinf.pstatic.net` should be a valid host")?;

    Ok(thumbnail)
}

// NOTE: only Originals have a banner.
fn banner(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .assumption("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .assumption(
            "`thmb>img`(banner) on `webtoons.com` originals Webtoon homepage is missing",
        )?
        .attr("src")
        .assumption("`src` attribute is missing in `.thmb>img` on `webtoons.com` originals Webtoon homepage")?;

    let mut banner = match Url::parse(url) {
        Ok(banner) => banner,
        Err(err) => assumption!(
            "banner url returned from `webtoons.com` Webtoon homepage was an invalid absolute path url: {err}\n\n{url}"
        ),
    };

    banner
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .assumption("`swebtoon-phinf.pstatic.net` should be a valid host")?;

    Ok(banner)
}

fn episode(element: &ElementRef<'_>, webtoon: &Webtoon) -> Result<Episode, Assumption> {
    let title = episode_title(element)?;

    let data_episode_no = element
        .value()
        .attr("data-episode-no")
        .assumption(
            "`data-episode-no` attribute should be found on `webtoons.com` Webtoon homepage, representing the episodes number",
        )?;

    let number = data_episode_no
        .parse::<u16>()
        .assumption_for(|err| format!("`data-episode-no` on `webtoons.com` should be parse into a `u16`, but got: {data_episode_no}: {err}"))?;

    let date = date(element, webtoon)?;

    Ok(Episode {
        webtoon: webtoon.clone(),
        title: Cache::new(title),
        number,
        published: Some(Published::from(date)),
        published_status: Some(PublishedStatus::Published),

        length: Cache::empty(),
        thumbnail: Cache::empty(),
        note: Cache::empty(),
        panels: Cache::empty(),
        views: None,
        // NOTE:
        // It is impossible to say from this page what its ad status, at any
        // point, could have been. In general, any random Original episode would
        // have been behind fast-pass, but the initial release episodes which never
        // were would be impossible to tell.
        //
        // Same goes for Canvas, impossible to say from just the info on this page.
        ad_status: None,
        top_comments: Cache::empty(),
    })
}

fn episode_title(html: &ElementRef<'_>) -> Result<String, Assumption> {
    let selector = Selector::parse("span.subj>span") //
        .assumption("`span.subj>span` should be a valid selector")?;

    let title = html
        .select(&selector)
        .next()
        .assumption("`span.subj>span` on `webtoons.com` Webtoon homepage should exist on page with episodes listed")?
        .text()
        .next()
        .assumption("`span.subj>span` on `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    assumption!(
        !title.is_empty(),
        "`webtoons.com` Webtoon hompeage episodes' title should never be empty"
    );

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn calculate_total_pages(html: &Html) -> Result<u16, WebtoonError> {
    let selector = Selector::parse("li._episodeItem>a>span.tx") //
        .assumption("`li._episodeItem>a>span.tx` should be a valid selector")?;

    // Counts the episodes listed per page. This is needed as there can be varying
    // amounts: 9 or 10, for example.
    let episodes_per_page =
        {
            let count = html.select(&selector).count();
            u16::try_from(count).assumption_for(|err| format!(
            "episodes per page count should be able to fit within a `u16`, got: {count}: {err}"
        ))?
        };

    let latest = html
        .select(&selector)
        .next()
        .assumption("`span.tx`(episodes) on `webtoons.com` Webtoon homepage was missing")?
        .text()
        .next()
        .assumption(
            "`span.tx`(episodes) on `webtoons.com` Webtoon homepage was found, but element was empty",
        )?
        .trim();

    assumption!(
        latest.starts_with('#'),
        "episode numbers on `webtoons.com` Webtoon homepages are prefixed with a `#`"
    );

    let episode = latest
        .trim_start_matches('#')
        .parse::<u16>()
        .assumption_for(|err| format!("the maximum amount of episodes we should realistically see should be able to fit in a `u16`, got: {latest}: {err}"))?;

    assumption!(episode > 0, "`webtoons.com` episode count starts at 1");

    Ok(episode.in_bucket_of(episodes_per_page))
}

pub(super) async fn episodes(webtoon: &Webtoon) -> Result<Vec<Episode>, WebtoonError> {
    let page = scrape(webtoon).await?;

    let pages = page.pages;

    // NOTE: currently all languages use this for the list element; this could change.
    let selector = Selector::parse("li._episodeItem") //
        .assumption("`li._episodeItem` should be a valid selector")?;

    let mut episodes = Vec::with_capacity(pages as usize * 10);

    for page in 1..=pages {
        let html = webtoon.client.webtoon_page(webtoon, Some(page)).await?;

        for element in html.select(&selector) {
            episodes.push(episode(&element, webtoon)?);
        }

        // Sleep for one second to prevent getting a 429 response code for going between the pages to quickly.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    assumption!(
        !episodes.is_empty(),
        "public facing webtoons on `webtoons.com` should always have at least one public episode"
    );

    match u16::try_from(episodes.len()) {
        Ok(_) => {}
        Err(err) => {
            assumption!(
                "`webtoons.com` Webtoons should never have more than 65,535 episodes: {err}\n\ngot: {}",
                episodes.len()
            )
        }
    }

    // NOTE: Consistently return by episode order
    episodes.sort_unstable_by(|a, b| a.number.cmp(&b.number));

    Ok(episodes)
}

pub(super) async fn first_episode(webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
    let page = scrape(webtoon).await?.pages;

    // NOTE: currently all languages use this for the list element; this could change.
    let selector = Selector::parse("li._episodeItem") //
        .assumption("`li._episodeItem` should be a valid selector")?;

    let html = webtoon.client.webtoon_page(webtoon, Some(page)).await?;

    let first = html
        .select(&selector)
        .next_back()
        .map(|element| episode(&element, webtoon))
        .transpose()?;

    match first {
        Some(first) => Ok(first),
        None => {
            assumption!(
                "`webtoons.com` Webtoon homepage should always have at least one episode for which to get a `first` episode"
            )
        }
    }
}

pub(super) async fn random_episode(webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
    let page = fastrand::u16(1..=scrape(webtoon).await?.pages);

    // NOTE: currently all languages use this for the list element; this could change.
    let selector = Selector::parse("li._episodeItem") //
        .assumption("`li._episodeItem` should be a valid selector")?;

    let html = webtoon.client.webtoon_page(webtoon, Some(page)).await?;

    let elements: Vec<ElementRef<'_>> = html.select(&selector).collect();

    assumption!(
        !elements.is_empty(),
        "`webtoons.com` Webtoon homepage should always have at least one episode for which to get an episode element for a `random` episode"
    );

    let idx = fastrand::usize(0..elements.len());
    let element = elements[idx];

    Ok(episode(&element, webtoon)?)
}

fn date(episode: &ElementRef<'_>, webtoon: &Webtoon) -> Result<NaiveDate, Assumption> {
    let selector = Selector::parse("span.date") //
        .assumption("`span.date` should be a valid selector")?;

    let text = episode
        .select(&selector)
        .next()
        .assumption("`span.date` should be found on `webtoons.com` Webtoon homepage with episodes listed on it")?
        .text()
        .next()
        .assumption("`span.date` on `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    let date = match webtoon.language() {
        Language::En => en::date(text)?,
        Language::Zh => todo!(),
        Language::Th => th::date(text)?,
        Language::Id => id::date(text)?,
        Language::Es => es::date(text)?,
        Language::Fr => fr::date(text)?,
        Language::De => de::date(text)?,
    };

    Ok(date)
}

/// Extracts creator usernames from the `div.author_area`.
///
/// This handles three specific `webtoons.com` quirks:
/// 1. Creators with accounts are inside `<a>` tags, while others are raw text nodes.
/// 2. The platform often inserts excessive tabs (`\t`) and newlines around names.
/// 3. Lists are terminated by a `...` text node or a language-specific "author info" button.
fn usernames(element: ElementRef<'_>) -> impl Iterator<Item = String> {
    element
        .children()
        // The "author info" button is always the last child.
        // We stop here so we don't accidentally parse the button's label as a username.
        .take_while(|node| {
            node.value()
                .as_element()
                // We stop if we hit an element node that is a button
                .is_none_or(|element| element.name() != "button")
        })
        // `webtoons` mixes <a> tags (accounts) and raw text (non-accounts).
        .filter_map(|node| {
            // If it's a text node, take it. If it's an element (like <a>), grab its inner text.
            node.value()
                .as_text()
                .map(|text| text.to_string())
                .or_else(|| {
                    ElementRef::wrap(node).map(|element| element.text().collect::<String>())
                })
        })
        // This is for cases where `webtoons.com`, for some ungodly reason, is
        // putting a bunch of tabs and newlines in the names, even if only one
        // creator.
        //
        // Examples:
        // - 66,666 Years: Advent of the Dark Mage
        // - Archie Comics: Big Ethel Energy
        // - Tower of God
        //
        // NOTE:
        // Creator can have a username with `,` in it.
        //     - https://www.webtoons.com/en/canvas/animals/list?title_no=738855
        //
        // To combat this, we end up splitting for `\t`(tab) as for stories
        // with multiple creators, there are (for some reason) tabs in the text:
        //     - `" SUMPUL , HereLee , Alphatart ... "`: https://www.webtoons.com/en/fantasy/the-remarried-empress/list?title_no=2135
        // This should allow commas in usernames, while still filtering away the standalone `,` separator.
        .flat_map(|text| {
            text.split('\t')
                .map(|str| str.trim().to_string())
                .collect::<Vec<_>>()
        })
        // Remove empty strings and the standalone comma separators
        // placed between creator links/nodes.
        .filter(|str| !str.is_empty() && str != ",")
        // For stories with many creators, `webtoons` appends a "..."
        // text node. We stop iterating here as no names follow this symbol.
        .take_while(|str| str != "...")
}

#[derive(Clone, Copy)]
enum Unit {
    Hundred = 100,
    Thousand = 1_000,
    Million = 1_000_000,
    Billion = 1_000_000_000,
}

/// A helper function that parses the counts(e.g. views and subscribers) on a Webtoons homepage.
fn count(
    input: &str,
    unit: Unit,
    separator: Option<&str>,
    suffix: Option<&str>,
) -> Result<u64, Assumption> {
    fn parse(prefix: &str, remainder: &str) -> Result<(u64, u64), Assumption> {
        let left = prefix.parse::<u64>()
                            .assumption_for(|err| {
                                 format!("`on the `webtoons.com` Webtoon homepage, prefix part of a count should always fit in a `u64`, got: {prefix}: {err}")
                             })?;

        let right = remainder.parse::<u64>()
                            .assumption_for(|err| {
                                format!("`on the `webtoons.com` Webtoon homepage, the remainder part of the count should always fit in a `u64`, got: {remainder}: {err}")
                            })?;

        Ok((left, right))
    }

    let number = match suffix {
        Some(suffix) => input.trim_end_matches(suffix),
        None => input,
    };

    let multiplier = unit as u64;

    match separator.map(|sep| number.split_once(sep)) {
        // Separator found (e.g., "1.5").
        Some(Some((prefix, remainder))) if matches!(unit, Unit::Million | Unit::Billion) => {
            let (left, right) = parse(prefix, remainder)?;
            assumption!(right < 10, "the `{right}` part of `{number}` should be less than '10', as the abbreviated million and billion numbers should only be single digit, i.e. 1..=9");
            Ok((left * multiplier) + (right * (multiplier / 10)))
        }

        // Separator found (e.g., "450,123").
        Some(Some((prefix, remainder))) if matches!(unit, Unit::Thousand) => {
            let (left, right) = parse(prefix, remainder)?;
            Ok((left * multiplier) + right)
        }

        // If separator failed to split(e.g., "1B" or "10M"), then can parse directly.
        //
        // Example:
        //     "1B" -> "1" -> 1 -> 1 * multiplier -> 1_000_000_000
        Some(None) => number
            .parse::<u64>()
            .map(|digit| digit * multiplier)
            .assumption_for(|err| {
                 format!("on the `webtoons.com` Webtoon homepage, a count without any separator should cleanly parse into single digit repesentation, got: {number}: {err}")
             }),

        // If no separator provided, then can parse directly.
        None => number
            .parse::<u64>()
            .assumption_for(|err| {
                format!("on the `webtoons.com` Webtoon homepage, a count without any separator should cleanly parse into single digit repesentation, got: {number}: {err}")
            }),

        Some(Some(_)) => assumption!("on `webtoons.com` Webtoon homepage, split `{number}` on `{separator:?}` but failed to match expected Thousand, Million, or Billion `matches!` arm"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_correct_counts() {
        // --- Billions ---
        assert_eq!(
            1_300_000_000,
            count("1.3B", Unit::Billion, Some("."), Some("B")).unwrap()
        );
        assert_eq!(
            1_000_000_000,
            count("1B", Unit::Billion, Some("."), Some("B")).unwrap()
        );
        assert_eq!(
            10_500_000_000, // Multi-digit leading part
            count("10.5B", Unit::Billion, Some("."), Some("B")).unwrap()
        );
        assert_eq!(
            1_200_000_000,
            count("1,2M", Unit::Billion, Some(","), Some("M")).unwrap()
        );

        // --- Millions ---
        assert_eq!(
            1_300_000,
            count("1.3M", Unit::Million, Some("."), Some("M")).unwrap()
        );
        assert_eq!(
            1_000_000,
            count("1M", Unit::Million, Some("."), Some("M")).unwrap()
        );
        assert_eq!(
            1_000_000, // Explicit zero decimal
            count("1.0M", Unit::Million, Some("."), Some("M")).unwrap()
        );
        assert_eq!(
            4_900_000,
            count("4,9M", Unit::Million, Some(","), Some("M")).unwrap()
        );
        assert_eq!(
            1_200_000,
            count("1,2JT", Unit::Million, Some(","), Some("JT")).unwrap()
        );

        // --- Thousands ---
        assert_eq!(
            112_362,
            count("112,362", Unit::Thousand, Some(","), None).unwrap()
        );
        assert_eq!(
            46_547,
            count("46.547", Unit::Thousand, Some("."), None).unwrap()
        );
        assert_eq!(
            1_005, // Comma with zero-padding in remainder
            count("1,005", Unit::Thousand, Some(","), None).unwrap()
        );
        assert_eq!(
            1_000, // Minimum thousand
            count("1,000", Unit::Thousand, Some(","), None).unwrap()
        );

        // --- Hundreds ---
        assert_eq!(999, count("999", Unit::Hundred, None, None).unwrap());
        assert_eq!(0, count("0", Unit::Hundred, None, None).unwrap());
        assert_eq!(1, count("1", Unit::Hundred, None, None).unwrap());
    }

    #[test]
    #[should_panic]
    fn should_fail_on_invalid_format() {
        // Example of a case that would break the logic (two decimal places)
        // If the assumption is 1 decimal place, "1.55B" would result in
        // (1 * 1B) + (55 * 100M) = 6.5B, which is wrong.
        //
        // This test ensures we know our current logic's limitations.
        let result = count("1.55B", Unit::Billion, Some("."), Some("B")).unwrap();
        assert_eq!(1_550_000_000, result);
    }
}
