use super::{super::episode::PublishedStatus, Page};
use crate::{
    platform::webtoons::{
        Client, Language, Type, Webtoon,
        creator::{self, Creator, Homepage},
        error::CreatorError,
        meta::{Genre, Scope},
        originals::Schedule,
        webtoon::{
            WebtoonError,
            episode::{Episode, Published},
        },
    },
    stdx::{
        cache::Cache,
        error::{Assume, AssumeFor, Assumption, assumption},
        math::MathExt,
    },
};
use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use std::str::FromStr;
use url::Url;

pub(super) async fn page(html: &Html, webtoon: &Webtoon) -> Result<Page, WebtoonError> {
    let page = match webtoon.scope {
        Scope::Original(_) => Page {
            title: title(html)?,
            creators: creators(html, &webtoon.client, webtoon.r#type()).await?,
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
            creators: creators(html, &webtoon.client, webtoon.r#type()).await?,
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

pub(super) async fn creators(
    html: &Html,
    client: &Client,
    r#type: Type,
) -> Result<Vec<Creator>, WebtoonError> {
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
            .assumption("`a.author` should be a valid selector")?;

        // Canvas creator must have a `webtoons.com` account.
        for selected in html.select(&selector) {
            let url = selected
                .value()
                .attr("href")
                .assumption("`href` is missing, `a.author` on a `webtoons.com` Canvas Webtoon homepage should always have one")?;

            let url = Url::parse(url)
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
                .map(|str| str.trim())
                .assumption("`webtoons.com` creator text element should always be populated")?;

            let creator = match creator::homepage(Language::En, profile, client).await {
                Ok(Some(Homepage {
                    id,
                    username,
                    followers,
                    has_patreon,
                })) => Creator {
                    client: client.clone(),
                    id: Some(id),
                    profile: Some(profile.into()),
                    username,
                    language: Language::En,
                    followers: Some(followers),
                    has_patreon: Some(has_patreon),
                },
                Ok(None) => Creator {
                    client: client.clone(),
                    id: None,
                    profile: Some(profile.into()),
                    username: username.to_string(),
                    language: Language::En,
                    followers: None,
                    has_patreon: None,
                },

                Err(CreatorError::Internal(err)) => return Err(WebtoonError::Internal(err)),
                Err(CreatorError::RequestFailed(err)) => {
                    return Err(WebtoonError::RequestFailed(err));
                }
                Err(CreatorError::UnsupportedLanguage) => {
                    assumption!("`Language::En` should be a supported language")
                }
                Err(CreatorError::InvalidCreatorProfile) => assumption!(
                    "creator profiles found on `webtoons.com` Webtoon homepage should be a valid profile"
                ),
            };

            assumption!(
                username == creator.username,
                "scraped creator username on `webtoons.com` Webtoon homepage should match the username found on the Creator homepage: found `{}`, expected `{}`",
                creator.username,
                username
            );

            creators.push(creator);

            // This check is needed as the creator limit is only for actual canvas
            // stories, and originals can have multiple creators, including creators
            // with accounts, which end up matching this selection.
            //
            // We still allow this loop to run even if on an Original, as this helps
            // to distinguish the `profile` field, always being `Some` for `webtoons.com`
            // accounts.
            if r#type == Type::Canvas {
                // NOTE: While this is saying that the loop will only run once, we
                // actually want to be informed if the platform can now have multiple
                // creators on canvas stories. This would be a big thing that we must
                // fix to accommodate!
                assumption!(
                    creators.len() == 1,
                    "`webtoons.com` canvas Webtoon homepages can only have one creator account associated with the Webtoon, got: {creators:?}"
                );
            }
        }
    }

    // Originals
    {
        let selector = Selector::parse(r"div.author_area") //
            .assumption("`div.author_area` should be a valid selector")?;

        // Originals creators that have no Webtoon account, or a mix of no accounts and `webtoons.com` accounts.
        if let Some(selected) = html.select(&selector).next() {
            for text in selected.text() {
                // The last text block in the element, meaning all creators have
                // been gone through.
                if text == "author info" {
                    break;
                }

                // This is for cases where `webtoons.com`, for some ungodly reason, is
                // putting a bunch of tabs and newlines in the names, even if only one
                // creator.
                //
                // Examples:
                // - 66,666 Years: Advent of the Dark Mage
                // - Archie Comics: Big Ethel Energy
                // - Tower of God
                //
                // NOTE: Creator can have a username with `,` in it.
                //     - https://www.webtoons.com/en/canvas/animals/list?title_no=738855
                // To combat this, we end up splitting for `\t`(tab) as for stories
                // with multiple creators, there are (for some reason) tabs in the text:
                //     - `" SUMPUL , HereLee , Alphatart ... "`: https://www.webtoons.com/en/fantasy/the-remarried-empress/list?title_no=2135
                // This should allow commas in usernames, while still filtering away the standalone `,` separator.
                'username: for username in text
                    .trim()
                    .split('\t')
                    .map(|str| str.trim())
                    .filter(|&str| str != ",")
                    // Last username in a multi-username scenario ends with ` ... `
                    .filter(|&str| str != "...")
                {
                    if username.is_empty() {
                        continue;
                    }

                    // `webtoons.com` creators have their name come up again in
                    // this loop. The text should be the exact same so it's safe
                    // to check if they already exist in the list, continuing to
                    // the next text block if so.
                    if creators.iter().any(|creator| creator.username == username) {
                        continue 'username;
                    }

                    creators.push(Creator {
                        client: client.clone(),
                        id: None,
                        profile: None,
                        username: username.to_string(),
                        language: Language::En,
                        followers: None,
                        has_patreon: None,
                    });
                }
            }
        }
    }

    assumption!(
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

pub(super) fn views(html: &Html) -> Result<u64, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .assumption("`em.cnt` should be a valid selector")?;

    let views = html
        .select(&selector)
        // First occurrence of `em.cnt` is for the views.
        .next()
        .assumption(
            "`em.cnt`(views) element is missing on english `webtoons.com` Webtoon homepage",
        )?
        .inner_html();

    assumption!(
        !views.is_empty(),
        "views element(`em.cnt`) on english `webtoons.com` Webtoon homepage should never be empty"
    );

    match views.as_str() {
        billions if billions.ends_with('B') => {
            let number = billions.trim_end_matches('B');

            match number.split_once('.') {
                Some((b, m)) => {
                    let billion = b.parse::<u64>()
                            .assumption_for(|err| format!("`on the english `webtoons.com` Webtoon homepage, the billions part of the views count should always fit in a `u64`, got: {b}: {err}"))?;

                    let hundred_million = m.parse::<u64>()
                            .assumption_for(|err| format!("`on the english `webtoons.com` Webtoon homepage, the hundred millions part of the views count should always fit in a `u64`, got: {m}: {err}"))?;

                    Ok((billion * 1_000_000_000) + (hundred_million * 100_000_000))
                }
                // If there is `1B`, this should just be `1` here, and thus we multiply by 1 billion.
                None => match number.parse::<u64>() {
                    Ok(digit) => Ok(digit * 1_000_000_000),
                    Err(err) => assumption!(
                        "on the english `webtoons.com` Webtoon homepage, a billion views without any `.` separator should cleanly parse into single digit repesentation, got: {number}: {err}"
                    ),
                },
            }
        }
        millions if millions.ends_with('M') => {
            let number = millions.trim_end_matches('M');

            match number.split_once('.') {
                Some((m, t)) => {
                    let million = m.parse::<u64>()
                        .assumption_for(|err| format!("`on the english `webtoons.com` Webtoon homepage, the millions part of the views count should always fit in a `u64`, got: {m}: {err}"))?;

                    let hundred_thousand = t.parse::<u64>()
                        .assumption_for(|err| format!("`on the english `webtoons.com` Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u64`, got: {t}: {err}"))?;

                    Ok((million * 1_000_000) + (hundred_thousand * 100_000))
                }
                // If there is `1M`, this should just be `1` here, and thus we multiply by 1 million.
                None => match number.parse::<u64>() {
                    Ok(digit) => Ok(digit * 1_000_000),
                    Err(err) => assumption!(
                        "on the english `webtoons.com` Webtoon homepage, a million views without any `.` separator should cleanly parse into `u64`, got: {number}: {err}"
                    ),
                },
            }
        }
        thousands if thousands.contains(',') => {
            let (thousandth, hundreth) = thousands
                .split_once(',')
                .with_assumption(|| format!("on `webtoons.com` english Webtoon homepage, < 1,000,000 views is always represented as a decimal value, eg. `469,035`, and so should always split on `,`, got: {thousands}"))?;

            let thousands = thousandth.parse::<u64>()
                .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the thousands part of the views count should always fit in a `u64`, got: {thousandth}: {err}"))?;

            let hundreds = hundreth.parse::<u64>()
                .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the hundred thousands part of the views count should always fit in a `u64`, got: {hundreth}: {err}"))?;

            Ok((thousands * 1_000) + hundreds)
        }
        hundreds => Ok(hundreds.parse::<u64>().assumption_for(|err| {
            format!("hundreds of views should fit in a `u64`, got: {hundreds}: {err}",)
        })?),
    }
}

pub(super) fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    let selector = Selector::parse(r"em.cnt") //
        .assumption("`em.cnt` should be a valid selector")?;

    let subscribers = html
        .select(&selector)
        // First instance of `em.cnt` is for views.
        .nth(1)
        .assumption("second instance of `em.cnt`(subscribers) on english `webtoons.com` Webtoon homepage is missing")?
        .inner_html();

    assumption!(
        !subscribers.is_empty(),
        "subscriber element(`em.cnt`) on english `webtoons.com` Webtoon homepage should never be empty"
    );

    match subscribers.as_str() {
        millions if millions.ends_with('M') => {
            match millions {
                float if float.contains('.') => {
                    let (millionth, hundred_thousandth) = millions
                        .trim_end_matches('M')
                        .split_once('.')
                        .assumption("on `webtoons.com` english Webtoon homepage, a million subscribers is always represented as a decimal value, with an `M` suffix, eg. `1.3M`, and so should always split on `.`")?;

                    let millions = millionth.parse::<u32>()
                        .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the millions part of the subscribers count should always fit in a `u32`, got: {millionth}: {err}"))?;

                    let hundred_thousands = hundred_thousandth.parse::<u32>()
                        .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundred_thousandth}: {err}"))?;

                    Ok((millions * 1_000_000) + (hundred_thousands * 100_000))
                }
                // Can be `1M` as well, with no decimal.
                digit =>Ok(digit
                    .trim_end_matches('M')
                    .parse::<u32>()
                    .map(|million| million * 1_000_000  )
                    .assumption_for(|err|format!(  "`webtoons.com` subscribers count ended with `M`, didn't have any `.` inside, which must mean its a whole number, yet failed to parse into a `u32`: {err}\n\n{digit}"))?),
            }
        }
        thousands if thousands.contains(',') => {
            let (thousandth, hundreth) = thousands
                .split_once(',')
                .with_assumption(|| format!("on `webtoons.com` english Webtoon homepage, < 1,000,000 subscribers is always represented as a decimal value, eg. `469,035`, and so should always split on `,`, got: {thousands}"))?;

            let thousands = thousandth.parse::<u32>()
                .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the thousands part of the subscribers count should always fit in a `u32`, got: {thousandth}: {err}"))?;

            let hundreds = hundreth.parse::<u32>()
                .assumption_for(|err| format!("`on the `webtoons.com` english Webtoon homepage, the hundred thousands part of the veiws count should always fit in a `u32`, got: {hundreth}: {err}"))?;

            Ok((thousands * 1_000) + hundreds)
        }
        hundreds => Ok(hundreds.parse::<u32>().assumption_for(|err| {
            format!("`0..999` should fit into a `u32`, got: {hundreds}: {err}")
        })?),
    }
}

// NOTE: Could also parse from the json on the story page `logParam`
// *ONLY* for Originals.
pub(super) fn schedule(html: &Html) -> Result<Schedule, WebtoonError> {
    let selector = Selector::parse(r"p.day_info") //
        .assumption("`p.day_info` should be a valid selector")?;

    let mut releases = Vec::new();

    for text in html
        .select(&selector)
        .next()
        .assumption(
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

    assumption!(
        !releases.is_empty(),
        "original Webtoon homepage on english `webtoons.com` should always have a release schedule, even if completed"
    );

    assumption!(
        releases.len() < 7,
        "original Webtoon homepage on english `webtoons.com` should always have 6 or less items, as if there was a list of 7 days, it would just say `daily` instead: `{releases:?}`"
    );

    let schedule = match Schedule::try_from(releases) {
        Ok(schedule) => schedule,
        Err(err) => assumption!(
            "english originals on `webtoons.com` should only have a few known release days/types: {err}"
        ),
    };

    Ok(schedule)
}

pub(super) fn summary(html: &Html) -> Result<String, WebtoonError> {
    let selector = Selector::parse(r"p.summary") //
        .assumption("`p.summary` should be a valid selector")?;

    let text = html
        .select(&selector)
        .next()
        .assumption("`p.summary`(summary) on english `webtoons.com` Webtoon homepage is missing")?
        .text()
        .next()
        .assumption("`p.summary` on english `webtoons.com` Webtoon homepage was found, but no text was present")?;

    let mut summary = String::new();

    // Gets rid of any weird formatting, such as newlines and tabs being in the middle of the summary.
    for word in text.split_whitespace() {
        summary.push_str(word);
        summary.push(' ');
    }

    // Removes the final spacing at the end while keeping it a string.
    summary.pop();

    assumption!(
        !summary.is_empty(),
        "english `webtoons.com` requires that the summary is not empty when creating/editing the Webtoon"
    );

    Ok(summary)
}

// NOTE: originals had their homepage thumbnails removed, so only canvas has one we can get.
pub(super) fn thumbnail(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .assumption("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .assumption(
            "`thmb>img`(thumbail) on english `webtoons.com` canvas Webtoon homepage is missing",
        )?
        .attr("src")
        .assumption("`src` attribute is missing in `.thmb>img` on english `webtoons.com` canvas Webtoon homepage")?;

    let mut thumbnail = match Url::parse(url) {
        Ok(thumbnail) => thumbnail,
        Err(err) => assumption!(
            "thumnbail url returned from english `webtoons.com` canvas Webtoon homepage was an invalid absolute path url: {err}\n\n{url}"
        ),
    };

    thumbnail
        // This host doesn't need a `referer` header to see the image.
        .set_host(Some("swebtoon-phinf.pstatic.net"))
        .assumption("`swebtoon-phinf.pstatic.net` should be a valid host")?;

    Ok(thumbnail)
}

// NOTE: only Originals have a banner.
pub(super) fn banner(html: &Html) -> Result<Url, WebtoonError> {
    let selector = Selector::parse(r".thmb>img") //
        .assumption("`.thmb>img` should be a valid selector")?;

    let url = html
        .select(&selector)
        .next()
        .assumption(
            "`thmb>img`(banner) on english `webtoons.com` originals Webtoon homepage is missing",
        )?
        .attr("src")
        .assumption("`src` attribute is missing in `.thmb>img` on english `webtoons.com` originals Webtoon homepage")?;

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

pub fn calculate_total_pages(html: &Html) -> Result<u16, WebtoonError> {
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

pub(super) fn episode(element: &ElementRef<'_>, webtoon: &Webtoon) -> Result<Episode, Assumption> {
    let title = episode_title(element)?;

    let data_episode_no = element
        .value()
        .attr("data-episode-no")
        .assumption(
            "`data-episode-no` attribute should be found on english `webtoons.com` Webtoon homepage, representing the episodes number",
        )?;

    let number = data_episode_no
        .parse::<u16>()
        .assumption_for(|err| format!("`data-episode-no` on english `webtoons.com` should be parse into a `u16`, but got: {data_episode_no}: {err}"))?;

    let date = date(element)?;

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

pub(super) fn episode_title(html: &ElementRef<'_>) -> Result<String, Assumption> {
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
        "english `webtoons.com` Webtoon hompeage episodes' title should never be empty"
    );

    Ok(html_escape::decode_html_entities(title).to_string())
}

fn date(episode: &ElementRef<'_>) -> Result<NaiveDate, Assumption> {
    let selector = Selector::parse("span.date") //
        .assumption("`span.date` should be a valid selector")?;

    let text = episode
        .select(&selector)
        .next()
        .assumption("`span.date` should be found on english `webtoons.com` Webtoon homepage with episodes listed on it")?
        .text()
        .next()
        .assumption("`span.date` on english `webtoons.com` Webtoon homepage should have text inside it")?
        .trim();

    // %b %e, %Y -> Jun 3, 2022
    // %b %d, %Y -> Jun 03, 2022
    // %F -> 2022-06-03 (ISO 8601)
    let date = NaiveDate::parse_from_str(text, "%b %e, %Y")
        .assumption_for(|err| format!("the english `webtoons.com` Webtoon homepage episode date should follow the `Jun 3, 2022` format, got: {text}: {err}"))?;

    Ok(date)
}
