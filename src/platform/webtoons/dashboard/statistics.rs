use scraper::{Html, Selector};

use crate::{
    platform::webtoons::{
        Webtoon,
        error::{StatsDashboardError, WebtoonError},
    },
    stdx::error::{Assume, AssumeFor, assumption},
};

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Stats {
    pub updates: u16,
    pub subscribers: u32,
    pub this_month: Current,
    pub last_month: Previous,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Current {
    pub updates: u8,
    pub monthly_views: u32,
    pub daily_views: u32,
    pub average_views_per_update: u32,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq, Default)]
pub struct Previous {
    pub updates: Option<u8>,
    pub monthly_views: Option<u32>,
    pub average_views_per_update: Option<u32>,
}

pub async fn scrape(webtoon: &Webtoon) -> Result<Stats, StatsDashboardError> {
    let html = webtoon.client.stats_dashboard(webtoon).await?;

    // TODO: For now only need subscribers from here, but could do the others as well.
    Ok(Stats {
        subscribers: subscribers(&html)?,
        ..Default::default()
    })
}

fn subscribers(html: &Html) -> Result<u32, WebtoonError> {
    {
        // NOTE:
        // This is a sanity check. As values on the page are all numbers,
        // there is no way to ensure that the column being gotten hasn't changed
        // order; we check the column name and make sure it aligns with the
        // value we are looking for.

        let selector = Selector::parse(r".col3>p") //
            .assumption("`.col3>p` should be a valid selector")?;

        let category = html
        .select(&selector)
        .next()
        .assumption("`.col3>p`, representing a category, is missing on `webtoons.com` Webtoon stats dashboard: should have an element which says what category its for, eg. `Subscribers`")?
        .text()
        .next()
        .assumption("`.col3>p` was found on `webtoons.com` Webtoon stats dashboard, which should have text that describes a category(Subscribers), but no text was present in element")?;

        assumption!(
            category == "Subscribers",
            "expected to find `Subscribers` category on `webtoons.com` stats dashboard at `.col3>p`, but instead found: `{category}`"
        );
    }

    let selector = Selector::parse(r".col3>.num") //
        .assumption("`.col3>.num` should be a valid selector")?;

    let count = html
        .select(&selector)
        .next()
        .assumption("`.col3>.num` on `webtoons.com` stats dashboard is missing: subscriber category was found, and should have a value associated with it, but nothing was found")?
        .text()
        .next()
        .assumption("`.col3>.num` on `webtoons.com` stats dashboard was found, but no text was present in element")?;

    let subscribers = match count {
        // NOTE:
        // It is *EXTREMELY* unlikely that a single Webtoon would have billions
        // of subscribers. Therefore, we just care about millions and below.

        // TODO: Haven't encountered a Webtoon to verify the actual shape this
        // would have with a million subscribers, but presumably it could very
        // well show the full count, without any abbreviations.
        millions if millions.ends_with('M') => {
            let (millionth, hundred_thousandth) = millions
                .trim_end_matches('M')
                .split_once('.')
                .assumption("on `webtoons.com` Webtoon homepage, a million subscribers is always represented as a decimal value, with an `M` suffix, eg. `1.3M`, and so should always split on `.`")?;

            let millions = millionth.parse::<u32>()
                .assumption_for(|err| format!("`on the `webtoons.com` Webtoon homepage, the millions part of the subscribers count should always fit in a `u32`, got: {millionth}: {err}"))?;

            let hundred_thousands = hundred_thousandth.parse::<u32>()
                .assumption_for(|err| format!("`on the `webtoons.com` Webtoon homepage, the hundred thousands part of the subscribers count should always fit in a `u32`, got: {hundred_thousandth}: {err}"))?;

            (millions * 1_000_000) + (hundred_thousands * 100_000)
        }
        // TODO: If the above `ends_with('M')` doesn't actually catch what would
        // be a million+ subscribers, and in-fact the count is fully shown, e.g
        // `1,123,394`, this would catch that branch.
        //
        // This could be a candidate to add an `assumption!` for, checking for
        // more than 1 `,`, but as this fallback handles that as well, leave for
        // now, until more is confirmed.
        count => count
            .replace(',', "")
            .parse::<u32>()
            .assumption_for(|err| format!("`on the `webtoons.com` Webtoon homepage, subscribers count should always fit in a `u32`, got: {count}: {err}"))?,
    };

    Ok(subscribers)
}
