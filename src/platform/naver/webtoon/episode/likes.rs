mod api;

use anyhow::Context;
use thiserror::Error;

use self::api::Api;
use crate::platform::webtoons::{meta::Scope, Client};

use super::{Episode, EpisodeError};

pub async fn for_episode(episode: &Episode) -> Result<u32, EpisodeError> {
    let response = episode
        .webtoon
        .client
        .get(&url(
            episode.webtoon.id,
            episode.webtoon.scope,
            episode.number,
        ))
        .await?
        .text()
        .await?;

    let api = Api::deserialize(&response).context(response)?;

    let api = api.contents.first().context(
        "`contents` field  in likes api didn't have a 0th element and it should always have one",
    )?;

    let likes = api
        .reactions
        .first()
        .map(|likes| likes.count)
        // NOTE: Because of the way the api returns responses, where even an episode not yet created can return
        // a 200 OK, the only fallback we can do here is to just return that there is `0` likes for the episode.
        //
        // If the response would have been instead a 404, we could have dealt with it in a way where the caller would
        // know that what happened and why there would be no value for it. Given that a new episode might not have the
        // expected `count` we also can't just return `None`, as in this context, returning a `0` would be more domain
        // accurate.
        .unwrap_or_default();

    Ok(likes)
}

fn url(id: u32, scope: Scope, episode: u16) -> String {
    let scope = match scope {
        Scope::Original(_) => "w",
        Scope::Canvas => "c",
    };

    format!("https://global.apis.naver.com/lineWebtoon/like_neoid/v1/search/contents?q=LINEWEBTOON[{scope}_{id}_{episode}]&pool=comic")
}
