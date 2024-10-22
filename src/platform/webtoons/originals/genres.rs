mod models;

use crate::{domain::webtoon::Genre, error::Error, http::WebtoonClient};
use ahash::AHashMap;
use arrayvec::ArrayVec;
use regex::Regex;
use scraper::{Html, Selector};
use std::str::FromStr;
use tracing::instrument;

#[instrument(name = "scraping genres page", skip_all)]
pub async fn scrape(client: &WebtoonClient) -> Result<AHashMap<u32, ArrayVec<Genre, 10>>, Error> {
    let html = client
        .get("https://www.webtoons.com/en/genre")
        .await?
        .html();

    let genre_types = parse_genre_types(&html)?;
    let genre_blocks = parse_genre_blocks_with_ids(&html)?;

    if genre_blocks.len() != genre_types.len() {
        return Err(Error::Unexpected(anyhow!(
            "not a 1:1 genre to genre block ratio: blocks `{}` : genres `{}`",
            genre_blocks.len(),
            genre_types.len()
        )));
    }

    let mut genres: AHashMap<u32, ArrayVec<Genre, 10>> = AHashMap::with_capacity(1500);

    for (genre, ids) in genre_types.iter().zip(genre_blocks) {
        for id in ids {
            genres
                .entry(id)
                .and_modify(|vec| vec.push(*genre))
                .or_insert_with(|| {
                    let mut vec = ArrayVec::new();
                    vec.push(*genre);
                    vec
                });
        }
    }

    Ok(genres)
}

#[instrument(name = "parsing all genre blocks", skip_all)]
fn parse_genre_blocks_with_ids(html: &Html) -> Result<Vec<Vec<u32>>> {
    let genre_list_selector = Selector::parse("ul.card_lst") //
        .expect("failed to parse genre list selector");

    let link_selector = Selector::parse("a.card_item") //
        .expect("failed to parse a tag link selector");

    // The genre block elements, one for each separate genre
    let blocks_elements = html.select(&genre_list_selector);

    // Each block will hold a list of Id's of stories within that block
    let mut blocks: Vec<Vec<u32>> = Vec::with_capacity(20);

    for cards in blocks_elements {
        // Holds the all the story id's in each block
        let mut block_ids: Vec<u32> = Vec::with_capacity(150);

        // For each link element, and thereby each story, found in each genre block, parse it's id.
        for url in cards.select(&link_selector) {
            let url = url
                .value()
                .attr("href")
                .with_context(|| "failed to get url from href attribute from url")?;

            let reg = Regex::new(r"title_no=(?P<id>\d+)").expect("incorrect regex pattern");

            let cap = reg
                .captures(url)
                .with_context(|| format!("failed to capture an id to parse from url: {url}"))?;

            let id = cap["id"]
                .parse::<u32>()
                .with_context(|| format!("failed to parse `{}` to u32", &cap["id"]))?;

            block_ids.push(id);
        }

        blocks.push(block_ids);
    }

    Ok(blocks)
}

#[instrument(name = "parsing genre types", skip_all)]
fn parse_genre_types(html: &Html) -> Result<Vec<Genre>> {
    let genre_selector = Selector::parse("h2.sub_title").expect("failed to parse genre selector");

    let mut genres = Vec::new();

    for genre_element in html.select(&genre_selector) {
        let genre = genre_element
            .text()
            .next()
            .with_context(|| "failed to get text from genre element")?;

        genres.push(Genre::from_str(genre)?);
    }

    Ok(genres)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use scraper::Html;

    const GENRES: &str = include_str!("./genres/test/originals_genres.html");

    #[test]
    fn should_get_block_ids() {
        let html = Html::parse_document(GENRES);

        let result: Vec<u32> = parse_genre_blocks_with_ids(&html)
            .unwrap()
            .into_iter()
            .flatten()
            .take(2)
            .collect();

        assert_eq!(result, vec![4572, 2135]);
    }

    #[test]
    fn should_get_genres() -> Result<()> {
        let html = Html::parse_document(GENRES);

        let result: Vec<Genre> = parse_genre_types(&html)?.into_iter().take(2).collect();

        assert_eq!(result, vec![Genre::Drama, Genre::Fantasy]);

        Ok(())
    }
}
