use assumptions::{Assume, Assumption, assume, assumption};
use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use std::{str::FromStr, time::Duration};
use url::Url;

use crate::{
    platform::webtoons::{
        Client, Type, Webtoon,
        creator::Creator,
        originals::Schedule,
        webtoon::{
            Genre, Scope,
            episode::{Published, PublishedStatus},
        },
    },
    stdx::cache::Cache,
};

use super::{WebtoonError, episode::Episode};

#[inline]
pub async fn scrape(webtoon: &Webtoon) -> Result<Homepage, WebtoonError> {
    let html = webtoon.client.fetch_webtoon_homepage(webtoon, None).await?;
    let homepage = Homepage::parse(&html, webtoon)?;
    Ok(homepage)
}

#[derive(Debug, Clone)]
pub struct Homepage {
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

impl Homepage {
    #[inline]
    fn parse(html: &Html, webtoon: &Webtoon) -> Result<Self, WebtoonError> {
        let page = match webtoon.scope {
            Scope::Original(_) => Self {
                title: title(html)?,
                creators: creators(html, &webtoon.client, webtoon)?,
                genres: genres(html)?,
                summary: summary(html)?,
                views: views(html)?,
                subscribers: subscribers(html)?,
                schedule: Some(schedule(html)?),
                thumbnail: None,
                banner: Some(banner(html)?),
                pages: calculate_total_pages(html)?,
            },
            Scope::Canvas => Self {
                title: title(html)?,
                creators: creators(html, &webtoon.client, webtoon)?,
                genres: genres(html)?,
                summary: summary(html)?,
                views: views(html)?,
                subscribers: subscribers(html)?,
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
        let homepage = self;
        &homepage.title
    }

    #[inline]
    pub(crate) fn creators(&self) -> &[Creator] {
        let homepage = self;
        &homepage.creators
    }

    #[inline]
    pub(crate) fn genres(&self) -> &[Genre] {
        let homepage = self;
        &homepage.genres
    }

    #[inline]
    pub(crate) fn summary(&self) -> &str {
        let homepage = self;
        &homepage.summary
    }

    #[inline]
    pub(crate) fn views(&self) -> u64 {
        let homepage = self;
        homepage.views
    }

    #[inline]
    pub(crate) fn subscribers(&self) -> u32 {
        let homepage = self;
        homepage.subscribers
    }

    #[inline]
    pub(crate) fn schedule(&self) -> Option<&Schedule> {
        let homepage = self;
        homepage.schedule.as_ref()
    }

    #[inline]
    pub(crate) fn thumbnail(&self) -> Option<&str> {
        let homepage = self;
        homepage.thumbnail.as_ref().map(|url| url.as_str())
    }

    #[inline]
    pub(crate) fn banner(&self) -> Option<&str> {
        let homepage = self;
        homepage.banner.as_ref().map(Url::as_str)
    }
}

#[inline]
fn title(html: &Html) -> Result<String, WebtoonError> {
    // `h1.subj` for originals, `h3.subj` for canvas.
    let selector = Selector::parse(r".subj") //
        .expect("`.subj` should be a valid selector");

    // The first occurrence of the element is the desired story's title.
    // The other instances are from recommended stories and are not wanted here.
    let selected = html
        .select(&selector) //
        .next()
        .assumption("`.subj` element should be present on `webtoons.com` Webtoon homepage")?;

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

#[inline]
fn creators(html: &Html, client: &Client, webtoon: &Webtoon) -> Result<Vec<Creator>, WebtoonError> {
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
    // Currently only Originals can have multiple creators for a single Webtoon.

    let mut creators = Vec::new();

    // Canvas
    let selector = Selector::parse(r"a.author") //
        .expect("`a.author` should be a valid selector");

    // Canvas creator must have a `webtoons.com` account.
    for selected in html.select(&selector) {
        let url = selected
                .value()
                .attr("href")
                .assumption("`a.author` element on `webtoons.com` Canvas Webtoon homepage should always have an `href` attribute")?;

        let url = Url::parse(url).assumption(
            "`href` attribute on `webtoons.com` Canvas Webtoon homepage should be a valid url",
        )?;

        // Creator profile should always be the last path segment:
        //     - https://www.webtoons.com/en/creator/792o8
        let profile = url
                .path_segments()
                .assumption("`href` url on `webtoons.com` Canvas Webtoon homepage should have path segments")?
                .next_back()
                .assumption("`href` url path segments on `webtoons.com` Canvas Webtoon homepage should have a last segment")?;

        // This is for cases where `webtoons.com`, for some ungodly reason, is
        // surrounding the name with a bunch of tabs and newlines, even if only
        // one creator.
        //
        // Examples:
        // - 66,666 Years: Advent of the Dark Mage
        // - Archie Comics: Big Ethel Energy
        // - Tower of God
        // - Press Play, Sami
        let username = selected.text().next().map(|this| this.trim()).assumption(
            "`a.author` element on `webtoons.com` Canvas Webtoon homepage should contain text",
        )?;

        let creator = Creator {
            client: client.clone(),
            username: username.to_string(),
            profile: Some(profile.into()),
            homepage: Cache::empty(),
        };

        creators.push(creator);
    }

    // Originals
    let selector = Selector::parse(r"div.author_area") //
        .expect("`div.author_area` should be a valid selector");

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
                    homepage: Cache::empty(),
                });
            }
        }
    }

    match webtoon.r#type() {
        Type::Original => assume!(
            !creators.is_empty(),
            "`webtoons.com` Webtoons must have some creator associated with them and displayed on their homepages"
        ),
        Type::Canvas => assume!(
            creators.len() == 1,
            "`webtoons.com` canvas Webtoon homepages should have exactly one creator account associated with the Webtoon, got: {creators:?}"
        ),
    }

    Ok(creators)
}

#[inline]
fn genres(html: &Html) -> Result<Vec<Genre>, WebtoonError> {
    // `h2.genre` for originals and `p.genre` for canvas.
    //
    // Doing just `.genre` gets the all instances of the class
    let selector = Selector::parse(r".info>.genre") //
        .expect("`.info>.genre` should be a valid selector");

    let mut genres = Vec::with_capacity(2);

    for selected in html.select(&selector) {
        let text = selected.text().next().assumption(
            "`.info>.genre` element on `webtoons.com` Webtoon homepage should contain text",
        )?;

        let genre = Genre::from_str(text)
            .assumption("`webtoons.com` Webtoon homepage should only contain recognized genres")?;

        genres.push(genre);
    }

    match genres.as_slice() {
        [_] | [_, _] => Ok(genres),
        [] => assumption!("`webtoons.com` Webtoon homepage should have at least one genre"),
        [_, _, _, ..] => {
            assumption!("`webtoons.com` Webtoon homepage should have at most two genres")
        }
    }
}

#[inline]
fn views(html: &Html) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let views = html
        .select(&selector)
        // First occurrence of `em.cnt` is for the views.
        .next()
        .assumption(
            "first `em.cnt` element on `webtoons.com` Webtoon homepage should be present for views",
        )?
        .inner_html();

    assume!(
        !views.is_empty(),
        "first `em.cnt` element on `webtoons.com` Webtoon homepage should never be empty"
    );

    let views = match views {
        billion if billion.ends_with('B') => count(&billion, Unit::Billion, Some('.'), Some('B')),
        million if million.ends_with('M') => count(&million, Unit::Million, Some('.'), Some('M')),
        thousand if thousand.contains(',') => count(&thousand, Unit::Thousand, Some(','), None),
        hundred => count(&hundred, Unit::Hundred, None, None),
    }?;

    Ok(views)
}

#[inline]
fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .expect("`em.cnt` should be a valid selector");

    let subscribers = html
        .select(&selector)
        // First instance of `em.cnt` is for views; second is for subscribers.
        .nth(1)
        .assumption("second `em.cnt` element on `webtoons.com` Webtoon homepage should be present for subscribers")?
        .inner_html();

    assume!(
        !subscribers.is_empty(),
        "second `em.cnt` element on `webtoons.com` Webtoon homepage should never be empty"
    );

    let subscribers = match subscribers {
        million if million.ends_with('M') => count(&million, Unit::Million, Some('.'), Some('M')),
        thousand if thousand.contains(',') => count(&thousand, Unit::Thousand, Some(','), None),
        hundred => count(&hundred, Unit::Hundred, None, None),
    }?
    .try_into()
    .assumption("subscribers count on `webtoons.com` Webtoon homepage should fit within a `u32`")?;

    Ok(subscribers)
}

#[inline]
fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info") //
        .expect("`p.day_info` should be a valid selector");

    let mut releases = Vec::new();

    for status in html
        .select(&selector)
        .next()
        .assumption(
            "`p.day_info` element should be present on `webtoons.com` Originals Webtoon homepage",
        )?
        .children()
        // Skip the <span> (icons like "UP")
        .filter_map(|node| node.value().as_text())
        .map(|text| text.trim())
        .find(|text| !text.is_empty())
        .map(|text| match text {
            "EVERYDAY" => text,
            _ => text.trim_start_matches("EVERY").trim_start(),
        })
        .assumption("`p.day_info` element on `webtoons.com` Originals Webtoon homepage should contain non-empty text")?
        .split_whitespace()
        // `MON,` -> `MON`
        .map(|text| text.trim_end_matches(','))
    {
        releases.push(status);
    }

    assume!(
        !releases.is_empty(),
        "`p.day_info` element on `webtoons.com` Originals Webtoon homepage should produce at least one release day"
    );

    assume!(
        releases.len() < 7,
        "`p.day_info` element on `webtoons.com` Originals Webtoon homepage should have at most 6 release days, got: `{releases:?}`"
    );

    let schedule = Schedule::try_from(releases).assumption(
        "`webtoons.com` Originals Webtoon homepage release schedule should only contain recognized days or types",
    )?;

    Ok(schedule)
}

#[inline]
fn summary(html: &Html) -> Result<String, WebtoonError> {
    let selector = Selector::parse(r"p.summary") //
        .expect("`p.summary` should be a valid selector");

    let text = html
        .select(&selector)
        .next()
        .assumption("`p.summary` element should be present on `webtoons.com` Webtoon homepage")?
        .text()
        .next()
        .assumption("`p.summary` element on `webtoons.com` Webtoon homepage should contain text")?;

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

#[inline]
fn thumbnail(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .expect("`.thmb>img` should be a valid selector");

    let url = html
        .select(&selector)
        .next()
        .assumption("`.thmb>img` element should be present on `webtoons.com` Webtoon homepage")?
        .attr("src")
        .assumption(
            "`.thmb>img` element on `webtoons.com` Webtoon homepage should have a `src` attribute",
        )?;

    let mut thumbnail = Url::parse(url).with_assumption(|| {
        format!("thumbnail url on `webtoons.com` Webtoon homepage should be a valid url: `{url}`")
    })?;

    thumbnail
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .expect("`swebtoon-phinf.pstatic.net` should be a valid url host");

    Ok(thumbnail)
}

#[inline]
fn banner(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .expect("`.thmb>img` should be a valid selector");

    let url = html
        .select(&selector)
        .next()
        .assumption(
            "`.thmb>img` element should be present on `webtoons.com` Originals Webtoon homepage",
        )?
        .attr("src")
        .assumption("`.thmb>img` element on `webtoons.com` Originals Webtoon homepage should have a `src` attribute")?;

    let mut banner = Url::parse(url).with_assumption(|| {
        format!(
            "banner url on `webtoons.com` Originals Webtoon homepage should be a valid url: `{url}`"
        )
    })?;

    banner
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .expect("`swebtoon-phinf.pstatic.net` should be a valid url host");

    Ok(banner)
}

#[inline]
fn episode(element: &ElementRef<'_>, webtoon: &Webtoon) -> Result<Episode, Assumption> {
    let title = episode_title(element)?;

    let data_episode_no = element
        .value()
        .attr("data-episode-no")
        .assumption(
            "`li` element on `webtoons.com` Webtoon episode list should always have a `data-episode-no` attribute",
        )?;

    let number = data_episode_no
        .parse::<u16>()
        .with_assumption(|| format!("`data-episode-no` attribute on `webtoons.com` Webtoon episode list should be parseable as a `u16`, got: `{data_episode_no}`"))?;

    let published = Published::from(date(element)?);

    assume!(
        published.year() >= 2014,
        "`webtoons.com` episode publish year should be 2014 or later, got: {}",
        published.year()
    );

    Ok(Episode {
        webtoon: webtoon.clone(),
        title: Cache::new(title),
        number,
        published: Some(published),
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

#[inline]
fn episode_title(html: &ElementRef<'_>) -> Result<String, Assumption> {
    let selector = Selector::parse("span.subj>span") //
        .expect("`span.subj>span` should be a valid selector");

    let title = html
        .select(&selector)
        .next()
        .assumption(
            "`span.subj>span` element should be present on `webtoons.com` Webtoon episode list",
        )?
        .text()
        .next()
        .assumption(
            "`span.subj>span` element on `webtoons.com` Webtoon episode list should contain text",
        )?
        .trim();

    assume!(
        !title.is_empty(),
        "`span.subj>span` element on `webtoons.com` Webtoon episode list should never be empty"
    );

    Ok(html_escape::decode_html_entities(title).to_string())
}

#[inline]
fn calculate_total_pages(html: &Html) -> Result<u16, WebtoonError> {
    let selector = Selector::parse("li._episodeItem>a>span.tx") //
        .expect("`li._episodeItem>a>span.tx` should be a valid selector");

    // Counts the episodes listed per page.
    //
    // This is needed as there can be varying amounts: 9 or 10, for example.
    let episodes_per_page = {
        let count = html.select(&selector).count();

        // WHY:
        // Page and episode counts on `webtoons.com` start at 1; an empty page must be page 1.
        if count == 0 {
            return Ok(1);
        }

        u16::try_from(count).assumption("episodes per page count should be fit within a `u16`")?
    };

    let latest = html
        .select(&selector)
        .next()
        .assumption("`span.tx` element should be present on `webtoons.com` Webtoon episode list")?
        .text()
        .next()
        .assumption("`span.tx` element on `webtoons.com` Webtoon episode list should contain text")?
        .trim();

    assume!(
        latest.starts_with('#'),
        "`webtoons.com` episode number should be prefixed with `#`, got: `{latest}`"
    );

    let episode = latest
        .trim_start_matches('#')
        .parse::<u16>()
        .with_assumption(|| {
            format!("`webtoons.com` episode number should be parseable as a `u16`, got: `{latest}`")
        })?;

    assume!(
        episode > 0,
        "`webtoons.com` episode numbers should start at 1, got: {episode}"
    );

    Ok(episode.div_ceil(episodes_per_page))
}

#[inline]
pub(super) async fn episodes(webtoon: &Webtoon) -> Result<Vec<Episode>, WebtoonError> {
    let page = scrape(webtoon).await?;
    let pages = page.pages;

    let selector = Selector::parse("li._episodeItem") //
        .expect("`li._episodeItem` should be a valid selector");

    let mut episodes = Vec::with_capacity(pages as usize * 10);

    for page in 1..=pages {
        let html = webtoon
            .client
            .fetch_webtoon_homepage(webtoon, Some(page))
            .await?;

        for element in html.select(&selector) {
            episodes.push(episode(&element, webtoon)?);
        }

        // Sleep for one second to prevent getting a 429 response code for going
        // between the pages too quickly.
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    assume!(
        !episodes.is_empty(),
        "public facing webtoons on `webtoons.com` should always have at least one public episode"
    );

    assume!(
        u16::try_from(episodes.len()).is_ok(),
        "`webtoons.com` Webtoon should not have more than 65,535 episodes, got: {}",
        episodes.len()
    );

    Ok(episodes)
}

#[inline]
pub(super) async fn first_episode(webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
    let page = scrape(webtoon).await?.pages;

    let selector = Selector::parse("li._episodeItem") //
        .expect("`li._episodeItem` should be a valid selector");

    let html = webtoon
        .client
        .fetch_webtoon_homepage(webtoon, Some(page))
        .await?;

    let first = html
        .select(&selector)
        .next_back()
        .map(|element| episode(&element, webtoon))
        .transpose()?
        .assumption("`webtoons.com` Webtoon homepage should always have at least one episode for which to get a `first` episode")?;

    Ok(first)
}

#[inline]
pub(super) async fn random_episode(webtoon: &Webtoon) -> Result<Episode, WebtoonError> {
    let page = fastrand::u16(1..=scrape(webtoon).await?.pages);

    let selector = Selector::parse("li._episodeItem") //
        .expect("`li._episodeItem` should be a valid selector");

    let html = webtoon
        .client
        .fetch_webtoon_homepage(webtoon, Some(page))
        .await?;

    let elements: Vec<ElementRef<'_>> = html.select(&selector).collect();

    assume!(
        !elements.is_empty(),
        "`webtoons.com` Webtoon episode list page should always have at least one episode element"
    );

    let idx = fastrand::usize(0..elements.len());

    let element = elements
        .get(idx)
        .expect("`idx` is within bounds of `elements` by construction");

    Ok(episode(element, webtoon)?)
}

#[inline]
fn date(episode: &ElementRef<'_>) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%b %e, %Y";

    let selector = Selector::parse("span.date") //
        .expect("`span.date` should be a valid selector");

    let text = episode
        .select(&selector)
        .next()
        .assumption("`span.date` element should be present on `webtoons.com` Webtoon episode list")?
        .text()
        .next()
        .assumption(
            "`span.date` element on `webtoons.com` Webtoon episode list should contain text",
        )?
        .trim();

    let date = NaiveDate::parse_from_str(text, FMT).with_assumption(|| {
        format!("`webtoons.com` Webtoon episode date should be parseable with format `{FMT}`, got: `{text}`")
    })?;

    Ok(date)
}

/// Extracts creator usernames from the `div.author_area`.
///
/// This handles three specific `webtoons.com` quirks:
/// 1. Creators with accounts are inside `<a>` tags, while others are raw text nodes.
/// 2. The platform often inserts excessive tabs (`\t`) and newlines around names.
/// 3. Lists are terminated by a `...` text node or a language-specific "author info" button.
#[inline]
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
#[inline]
fn count(
    input: &str,
    unit: Unit,
    separator: Option<char>,
    suffix: Option<char>,
) -> Result<u64, Assumption> {
    fn parse(prefix: &str, remainder: &str) -> Result<(u64, u64), Assumption> {
        let left = prefix
            .parse::<u64>()
            .with_assumption(|| {
                format!("prefix part of a count on `webtoons.com` Webtoon homepage should be parseable as a `u64`, got: `{prefix}`")
            })?;

        let right = remainder
            .parse::<u64>()
            .with_assumption(|| {
                format!("remainder part of a count on `webtoons.com` Webtoon homepage should be parseable as a `u64`, got: `{remainder}`")
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
            assume!(
                right < 10,
                "fractional part of abbreviated count on `webtoons.com` Webtoon homepage should be a single digit, got: `{right}` in `{number}`"
            );
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
            .with_assumption(|| {
                 format!("count without separator on `webtoons.com` Webtoon homepage should be parseable as a `u64`, got: `{number}`")
             }),

        // If no separator provided, then can parse directly.
        None => number
            .parse::<u64>()
            .with_assumption(|| {
                format!("count without separator on `webtoons.com` Webtoon homepage should be parseable as a `u64`, got: `{number}`")
            }),

        Some(Some(_)) => assumption!("`webtoons.com` Webtoon homepage count `{number}` should match a known unit (Thousand, Million, or Billion), got unexpected separator split"),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn should_parse_correct_counts() {
        // --- Billions ---
        assert_eq!(
            1_300_000_000,
            count("1.3B", Unit::Billion, Some('.'), Some('B')).unwrap()
        );
        assert_eq!(
            1_000_000_000,
            count("1B", Unit::Billion, Some('.'), Some('B')).unwrap()
        );
        assert_eq!(
            10_500_000_000, // Multi-digit leading part
            count("10.5B", Unit::Billion, Some('.'), Some('B')).unwrap()
        );

        // --- Millions ---
        assert_eq!(
            1_300_000,
            count("1.3M", Unit::Million, Some('.'), Some('M')).unwrap()
        );
        assert_eq!(
            1_000_000,
            count("1M", Unit::Million, Some('.'), Some('M')).unwrap()
        );
        assert_eq!(
            1_000_000, // Explicit zero decimal
            count("1.0M", Unit::Million, Some('.'), Some('M')).unwrap()
        );

        // --- Thousands ---
        assert_eq!(
            112_362,
            count("112,362", Unit::Thousand, Some(','), None).unwrap()
        );
        assert_eq!(
            1_005, // Comma with zero-padding in remainder
            count("1,005", Unit::Thousand, Some(','), None).unwrap()
        );
        assert_eq!(
            1_000, // Minimum thousand
            count("1,000", Unit::Thousand, Some(','), None).unwrap()
        );

        // --- Hundreds ---
        assert_eq!(999, count("999", Unit::Hundred, None, None).unwrap());
        assert_eq!(0, count("0", Unit::Hundred, None, None).unwrap());
        assert_eq!(1, count("1", Unit::Hundred, None, None).unwrap());
    }

    #[test]
    #[should_panic(
        expected = "`right < 10`: fractional part of abbreviated count on `webtoons.com` Webtoon homepage should be a single digit, got: `55` in `1.55`"
    )]
    fn should_fail_on_invalid_format() {
        // Example of a case that would break the logic (two decimal places)
        // If the assumption is 1 decimal place, "1.55B" would result in
        // (1 * 1B) + (55 * 100M) = 6.5B, which is wrong.
        //
        // This test ensures we know our current logic's limitations.
        let result = count("1.55B", Unit::Billion, Some('.'), Some('B')).unwrap();
        assert_eq!(1_550_000_000, result);
    }
}
