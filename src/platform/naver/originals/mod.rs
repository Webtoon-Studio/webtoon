//! Represents an abstraction for the `https://www.webtoons.com/*/originals` endpoint.

// mod genres;
use anyhow::Context;
use scraper::{Html, Selector};

use super::{errors::OriginalsError, Client, Webtoon};

pub(super) async fn scrape(client: &Client) -> Result<Vec<Webtoon>, OriginalsError> {
    // NOTE: Currently all languages follow this pattern
    let selector = Selector::parse("ul.daily_card>li>a") //
        .expect("`ul.daily_card>li>a` should be a valid selector");

    let mut webtoons = Vec::with_capacity(1000);

    let document = client.originals_page(language).await?.text().await?;

    let html = Html::parse_document(&document);

    // TODO: Need to access the `genres` page as the home page of the webtoons only contains one genre when there could
    // be many more.
    // let genres = genres::scrape(client).await?;

    for card in html.select(&selector) {
        let href = card
            .attr("href")
            .context("`href` is missing, `a` tag should always have one")?;

        webtoons.push(Webtoon::from_url_with_client(href, client)?);
    }

    Ok(webtoons)
}
