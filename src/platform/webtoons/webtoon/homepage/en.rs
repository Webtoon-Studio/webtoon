use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
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
    stdx::{
        cache::Cache,
        error::{Invariant, invariant},
        math::MathExt,
    },
};

use super::{
    super::episode::{self, PublishedStatus},
    Page,
};

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
            thumbnail: Some(thumbnail(html)?),
            banner: Some(banner(html)?),
            pages: calculate_total_pages(html)?,
        },
    };

    Ok(page)
}

pub(super) fn title(html: &Html) -> Result<String, WebtoonError> {
    // `h1.subj` for originals, `h3.subj` for canvas.
    let selector = Selector::parse(r".subj") //
        .invariant("`.subj` should be a valid selector")?;

    // The first occurrence of the element is the desired story's title.
    // The other instances are from recommended stories and are not wanted here.
    let selected = html
        .select(&selector) //
        .next()
        .invariant("`.subj`(title) is missing on `webtoons.com` Webtoon homepage")?;

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

pub(super) fn creators(html: &Html, client: &Client) -> Result<Vec<Creator>, WebtoonError> {
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
    {
        let selector = Selector::parse(r"a.author") //
            .invariant("`a.author` should be a valid selector")?;

        // Canvas creator must have a `webtoons.com` account.
        for selected in html.select(&selector) {
            let url = selected
                .value()
                .attr("href")
                .invariant("`href` is missing, `a.author` on a `webtoons.com` Canvas Webtoon homepage should always have one")?;

            let url = Url::parse(url)
                .invariant("`webtoons.com` canvas Webtoon homepage should always return valid and wellformed urls")?;

            // Creator profile should always be the last path segment:
            //     - https://www.webtoons.com/en/creator/792o8
            let profile = url
                .path_segments()
                .invariant("`href` attribute on `webtoons.com` canvas Webtoon homepage should have path segments")?
                .next_back()
                .invariant("Creator homepage url on `webtoons.com` Canvas homepage had segments, but `next_back` failed")?;

            let mut username = String::new();

            // This is for cases where `webtoons.com`, for some ungodly reason is
            // putting a bunch of tabs and new-lines in the names.
            //
            // Examples:
            // - 66,666 Years: Advent of the Dark Mage
            // - Archie Comics: Big Ethel Energy
            // - Tower of God
            for text in selected.text() {
                for text in text.split_whitespace() {
                    username.push_str(text);
                    username.push(' ');
                }

                // Remove trailing space from end of last iteration.
                username.pop();
            }

            creators.push(Creator {
                client: client.clone(),
                language: Language::En,
                profile: Some(profile.into()),
                username,
                homepage: Arc::new(RwLock::new(None)),
            });

            // NOTE: While this is saying that the loop will only run once, we
            // actually want to be informed if the platform can now have multiple
            // creators on canvas stories. This would be a big thing that we must
            // fix to accommodate!
            invariant!(
                creators.len() == 1,
                "`webtoons.com` canvas Webtoon homepages can only have one creator account associated with the Webtoon, got: {creators:?}"
            );
        }
    }

    // Originals
    {
        let selector = Selector::parse(r"div.author_area") //
            .invariant("`div.author_area` should be a valid selector")?;

        // Originals creators that have no Webtoon account, or a mix of no accounts and `webtoons.com` accounts.
        if let Some(selected) = html.select(&selector).next() {
            for text in selected.text() {
                // The last text block in the element, meaning all creators have
                // been gone through.
                if text == "author info" {
                    break;
                }

                'username: for username in text.split(',') {
                    let username = username.trim().trim_end_matches("...").trim();
                    if username.is_empty() {
                        continue;
                    }

                    // `webtoons.com` creators have their name come up again in
                    // this loop. The text should be the exact same so it's safe
                    // to check if they already exist in the list, continuing to
                    // the next text block if so.
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
                        homepage: Arc::new(RwLock::new(None)),
                    });
                }
            }
        }
    }

    invariant!(
        !creators.is_empty(),
        "`webtoons.com` Webtoons must have some creator associated with them and displayed on their homepages"
    );

    Ok(creators)
}

pub(super) fn genres(html: &Html) -> Result<Vec<Genre>, WebtoonError> {
    // `h2.genre` for originals and `p.genre` for canvas.
    //
    // Doing just `.genre` gets the all instances of the class
    let selector = Selector::parse(r".info>.genre") //
        .invariant("`.info>.genre` should be a valid selector")?;

    let mut genres = Vec::with_capacity(2);

    for selected in html.select(&selector) {
        let text = selected
            .text()
            .next()
            .invariant(" the `.info>.genre` tag was found on `webtoons.com` Webtoon homepage, but no text was present inside the element")?;

        let genre = match Genre::from_str(text) {
            Ok(genre) => genre,
            Err(err) => {
                invariant!("`webtoons.com` Webtoon homepage had an unexpected genre: {err}")
            }
        };

        genres.push(genre);
    }

    match genres.as_slice() {
        [_] | [_, _] => Ok(genres),
        [] => invariant!("no genre was found on `webtoons.com` Webtoon homepage"),
        [_, _, _, ..] => {
            invariant!("more than two genres were found on `webtoons.com` Webtoon homepage")
        }
    }
}

pub(super) fn views(html: &Html) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .invariant("`em.cnt` should be a valid selector")?;

    let views = html
        .select(&selector)
        // First occurrence of `em.cnt` is for the views.
        .next()
        .invariant("`em.cnt`(views) element is missing on english `webtoons.com` Webtoon homepage")?
        .inner_html();

    invariant!(
        !views.is_empty(),
        "views element(`em.cnt`) on english `webtoons.com` Webtoon homepage should never be empty"
    );

    match views.as_str() {
        billions if billions.ends_with('B') => {
            let number = billions.trim_end_matches('B');

            match number.split_once('.') {
                    Some((b, m)) => {
                        let billion = b.parse::<u64>()
                            .invariant(format!("`on the english `webtoons.com` Webtoon homepage, the billions part of the views count should always fit in a `u64`, got: {b}"))?;

                        let hundred_million = m.parse::<u64>()
                            .invariant(format!("`on the english `webtoons.com` Webtoon homepage, the hundred millions part of the views count should always fit in a `u64`, got: {m}"))?;

                        Ok((billion * 1_000_000_000) + (hundred_million * 100_000_000))
                    },
                    None => Ok(number.parse::<u64>()
                        .invariant(format!("on the english `webtoons.com` Webtoon homepage, a billion views without any `.` separator should cleanly parse into `u64`, got: {number}"))?),
                }

        }
        millions if millions.ends_with('M') => {
            let number  = millions.trim_end_matches('M');

            match number.split_once('.') {
                Some((m, t)) => {
                    let million = m.parse::<u64>()
                        .invariant(format!("`on the english `webtoons.com` Webtoon homepage, the millions part of the views count should always fit in a `u64`, got: {m}"))?;

                    let hundred_thousand = t.parse::<u64>()
                        .invariant(format!("`on the english `webtoons.com` Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u64`, got: {t}"))?;

                    Ok((million * 1_000_000) + (hundred_thousand * 100_000))
                },
                None => Ok(number.parse::<u64>()
                        .invariant(format!("on the english `webtoons.com` Webtoon homepage, a million views without any `.` separator should cleanly parse into `u64`, got: {number}"))?),
            }


        }
        // PERF: refactor to use splits like `subscribers` below has. No string allocation, and can branch on thousands and hundreds separately
        thousands_or_less => Ok(thousands_or_less.replace(',', "").parse::<u64>().invariant(format!(
            "hundreds to hundreds of thousands of views should fit in a `u64`, got: {thousands_or_less}",
        ))?),
    }
}

pub(super) fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .invariant("`em.cnt` should be a valid selector")?;

    let subscribers = html
        .select(&selector)
        // First instance of `em.cnt` is for views.
        .nth(1)
        .invariant("second instance of `em.cnt`(subscribers) on english `webtoons.com` Webtoon homepage is missing")?
        .inner_html();

    invariant!(
        !subscribers.is_empty(),
        "subscriber element(`em.cnt`) on english `webtoons.com` Webtoon homepage should never be empty"
    );

    match subscribers.as_str() {
        millions if millions.ends_with('M') => {
            let (millionth, hundred_thousandth) = millions
                .trim_end_matches('M')
                .split_once('.')
                .invariant("on `webtoons.com` english Webtoon homepage, a million subscribers is always represented as a decimal value, with an `M` suffix, eg. `1.3M`, and so should always split on `.`")?;

            let millions = millionth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` english Webtoon homepage, the millions part of the subscribers count should always fit in a `u32`, got: {millionth}"))?;

            let hundred_thousands = hundred_thousandth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` english Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundred_thousandth}"))?;

            Ok((millions * 1_000_000) + (hundred_thousands * 100_000))
        }
        thousands if thousands.contains(',') => {
            let (thousandth, hundreth) = thousands
                .split_once(',')
                .invariant(format!("on `webtoons.com` english Webtoon homepage, < 1,000,000 subscribers is always represented as a decimal value, eg. `469,035`, and so should always split on `,`, got: {thousands}"))?;

            let thousands = thousandth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` english Webtoon homepage, the thousands part of the subscribers count should always fit in a `u32`, got: {thousandth}"))?;

            let hundreds = hundreth.parse::<u32>()
                .invariant(format!("`on the `webtoons.com` english Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundreth}"))?;

            Ok((thousands * 1_000) + hundreds)
        }
        hundreds => Ok(hundreds
            .parse::<u32>()
            .invariant(format!("`0..999` should fit into a `u32`, got: {hundreds}"))?),
    }
}

// NOTE: Could also parse from the json on the story page `logParam`
// *ONLY* for Originals.
pub(super) fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info") //
        .invariant("`p.day_info` should be a valid selector")?;

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .invariant(
            "`p.day_info`(schedule) on english `webtoons.com` originals Webtoons is missing",
        )?
        .text()
    {
        // `UP` icon text.
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

    invariant!(
        !releases.is_empty(),
        "original Webtoon homepage on english `webtoons.com` should always have a release schedule, even if completed"
    );

    let schedule = match Schedule::try_from(releases) {
        Ok(schedule) => schedule,
        Err(err) => invariant!(
            "english originals on `webtoons.com` should only have a few known release days/types: {err}"
        ),
    };

    Ok(schedule)
}

pub(super) fn summary(html: &Html) -> Result<String, WebtoonError> {
    let selector = Selector::parse(r"p.summary") //
        .invariant("`p.summary` should be a valid selector")?;

    let text = html
        .select(&selector)
        .next()
        .invariant("`p.summary`(summary) on english `webtoons.com` Webtoon homepage is missing")?
        .text()
        .next()
        .invariant("`p.summary` on english `webtoons.com` Webtoon homepage was found, but no text was present")?;

    let mut summary = String::new();

    // Gets rid of any weird formatting, such as newlines and tabs being in the middle of the summary.
    for word in text.split_whitespace() {
        summary.push_str(word);
        summary.push(' ');
    }

    // Removes the final spacing at the end while keeping it a string.
    summary.pop();

    invariant!(
        !summary.is_empty(),
        "english `webtoons.com` requires that the summary is not empty when creating/editing the Webtoon"
    );

    Ok(summary)
}

// NOTE: originals had their homepage thumbnails removed, so only canvas has one we can get.
pub(super) fn thumbnail(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .invariant("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .invariant(
            "`thmb>img`(thumbail) on english `webtoons.com` canvas Webtoon homepage is missing",
        )?
        .attr("src")
        .invariant("`src` attribute is missing in `.thmb>img` on english `webtoons.com` canvas Webtoon homepage")?;

    let mut thumbnail = Url::parse(url)
        .invariant("thumnbail url returned from english `webtoons.com` canvas Webtoon homepage was an invalid absolute path url")?;

    thumbnail
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .invariant("`swebtoon-phinf.pstatic.net` should be a valid host")?;

    Ok(thumbnail)
}

// NOTE: only Originals have a banner.
pub(super) fn banner(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .invariant("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .invariant(
            "`thmb>img`(banner) on english `webtoons.com` originals Webtoon homepage is missing",
        )?
        .attr("src")
        .invariant("`src` attribute is missing in `.thmb>img` on english `webtoons.com` originals Webtoon homepage")?;

    let mut banner = Url::parse(url).invariant(
        "banner url returned from `webtoons.com` Webtoon homepage was an invalid absolute path url",
    )?;

    banner
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .invariant("`swebtoon-phinf.pstatic.net` should be a valid host")?;

    Ok(banner)
}

pub fn calculate_total_pages(html: &Html) -> Result<u16, WebtoonError> {
    let selector = Selector::parse("li._episodeItem>a>span.tx") //
        .invariant("`li._episodeItem>a>span.tx` should be a valid selector")?;

    // Counts the episodes listed per page. This is needed as there can be varying
    // amounts: 9 or 10, for example.
    let episodes_per_page = {
        let count = html.select(&selector).count();
        u16::try_from(count).invariant(format!(
            "episodes per page count should be able to fit within a `u16`, got: {count}"
        ))?
    };

    let latest = html
        .select(&selector)
        .next()
        .invariant("`span.tx`(episodes) on `webtoons.com` Webtoon homepage was missing")?
        .text()
        .next()
        .invariant(
            "`span.tx`(episodes) on `webtoons.com` Webtoon homepage was found, but element was empty",
        )?
        .trim();

    invariant!(
        latest.starts_with('#'),
        "episode numbers on `webtoons.com` Webtoon homepages are prefixed with a `#`"
    );

    let episode = latest
        .trim_start_matches('#')
        .parse::<u16>()
        .invariant(format!("the maximum amount of episodes we should realistically see should be able to fit in a `u16`, got: {latest}"))?;

    invariant!(episode > 0, "`webtoons.com` episode count starts at 1");

    Ok(episode.in_bucket_of(episodes_per_page))
}

pub(super) fn episode(
    element: &ElementRef<'_>,
    webtoon: &Webtoon,
) -> Result<Episode, WebtoonError> {
    let title = episode_title(element)?;

    let data_episode_no = element
        .value()
        .attr("data-episode-no")
        .invariant(
            "`data-episode-no` attribute should be found on english `webtoons.com` Webtoon homepage, representing the episodes number",
        )?;

    let number = data_episode_no
        .parse::<u16>()
        .invariant(format!("`data-episode-no` on english `webtoons.com` should be parse into a `u16`, but got: {data_episode_no}"))?;

    let published = date(element)?;

    Ok(Episode {
        webtoon: webtoon.clone(),
        season: Cache::new(episode::season(&title)?),
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
    })
}

pub(super) fn episode_title(html: &ElementRef<'_>) -> Result<String, WebtoonError> {
    let selector = Selector::parse("span.subj>span") //
        .invariant("`span.subj>span` should be a valid selector")?;

    let title = html
        .select(&selector)
        .next()
        .invariant("`span.subj>span` on `webtoons.com` Webtoon homepage should exist on page with episodes listed")?
        .text()
        .next()
        .invariant("`span.subj>span` on `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    invariant!(
        !title.is_empty(),
        "english `webtoons.com` Webtoon hompeage episodes' title should never be empty"
    );

    Ok(html_escape::decode_html_entities(title).to_string())
}

// NOTE: Currently forces all dates to be at 02:00 UTC as that's when Originals
// get released. For more accurate times, must have a session.
fn date(episode: &ElementRef<'_>) -> Result<DateTime<Utc>, WebtoonError> {
    let selector = Selector::parse("span.date") //
        .invariant("`span.date` should be a valid selector")?;

    let text = episode
        .select(&selector)
        .next()
        .invariant("`span.date` should be found on english `webtoons.com` Webtoon homepage with episodes listed on it")?
        .text()
        .next()
        .invariant("`span.date` on english `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    // %b %e, %Y -> Jun 3, 2022
    // %b %d, %Y -> Jun 03, 2022
    // %F -> 2022-06-03 (ISO 8601)
    let date = NaiveDate::parse_from_str(text, "%b %e, %Y")
        .invariant(format!("the english `webtoons.com` Webtoon homepage episode date should follow the `Jun 3, 2022` format, got: {text}"))?;

    let time =
        NaiveTime::from_hms_opt(2, 0, 0).invariant("2:00:00 should be a valid `NaiveTime`")?;

    Ok(DateTime::from_naive_utc_and_offset(
        date.and_time(time),
        Utc,
    ))
}
