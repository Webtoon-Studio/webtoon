#![allow(clippy::ignore_without_reason)]
use webtoon::platform::webtoons::{Client, Language, canvas::Sort, error::CreatorError};

#[tokio::test]
#[ignore]
async fn canvas() {
    let client = Client::new();

    let page = fastrand::u16(1..=5000);

    let canvas = client
        .canvas(Language::En, page..=page, Sort::Popularity)
        .await
        .unwrap_or_else(|err| panic!("failed on page {page}: {err}"));

    assert_eq!(20, canvas.len());

    for webtoon in canvas {
        eprintln!("\nChecking `Canvas` Webtoon {}", webtoon.id());

        let _title = webtoon.title().await.unwrap();

        let creators = webtoon.creators().await.unwrap();
        match creators.as_slice() {
            [] => unreachable!("every webtoon must have a creator"),
            [_first, _second, _rest @ ..] => {
                unreachable!("canvas stories can only have one creator: {creators:?}")
            }
            [creator] => {
                match creator.has_patreon().await {
                    Ok(_)
                    | Err(
                        CreatorError::InvalidCreatorProfile | CreatorError::PageDisabledByCreator,
                    ) => {}
                    Err(err) => panic!("{err}"),
                }
                match creator.id().await {
                    Ok(_)
                    | Err(
                        CreatorError::InvalidCreatorProfile | CreatorError::PageDisabledByCreator,
                    ) => {}
                    Err(err) => panic!("{err}"),
                }
            }
        }

        let genres = webtoon.genres().await.unwrap();
        match genres.as_slice() {
            [] => unreachable!("every webtoons must have a genre"),
            [_genre] => {}
            // Can only have two genres assigned
            [_first, _second, rest @ ..] => {
                assert!(rest.is_empty());
            }
        }

        let thumbnail = webtoon.thumbnail().await.unwrap();
        assert!(thumbnail.is_some());

        let banner = webtoon.banner().await.unwrap();
        assert!(banner.is_none());

        let language = webtoon.language();
        assert_eq!(Language::En, language);

        let schedule = webtoon.schedule().await.unwrap();
        assert!(schedule.is_none());

        let _views = webtoon.views().await.unwrap();

        let _subscribers = webtoon.subscribers().await.unwrap();

        let _summary = webtoon.summary().await.unwrap();

        let episode = webtoon.random_episode().await.unwrap();
        eprintln!("episode {}", episode.number());

        let _title = episode.title().await.unwrap();
        let _season = episode.season().await.unwrap();
        let _thumbnail = episode.thumbnail().await.unwrap();
        let _length = episode.length().await.unwrap();
        let _likes = episode.likes().await.unwrap();

        episode
            .posts_for_each(async |post| {
                let _replies = post.replies().await.unwrap();
            })
            .await
            .unwrap();
    }
}

#[tokio::test]
#[ignore]
async fn originals() {
    let client = Client::new();

    let originals = client.originals(Language::En).await.unwrap();

    assert!(!originals.is_empty());

    for _ in 1..=10 {
        let idx = fastrand::usize(0..originals.len());
        let webtoon = &originals[idx];

        eprintln!("\nChecking `Original` Webtoon {}", webtoon.id());

        let _title = webtoon.title().await.unwrap();

        let creators = webtoon.creators().await.unwrap();
        match creators.as_slice() {
            [] => unreachable!("every webtoon must have a creator"),
            creators => {
                for creator in creators {
                    // TODO: Need to verify that this works for Korean Creators.
                    match creator.has_patreon().await {
                        Ok(_)
                        | Err(
                            CreatorError::PageDisabledByCreator
                            | CreatorError::InvalidCreatorProfile,
                        ) => {}
                        Err(err) => panic!("{err}"),
                    }

                    // TODO: Need to verify that this works for Korean Creators.
                    match creator.id().await {
                        Ok(_)
                        | Err(
                            CreatorError::PageDisabledByCreator
                            | CreatorError::InvalidCreatorProfile,
                        ) => {}
                        Err(err) => panic!("{err}"),
                    }
                }
            }
        }

        let genres = webtoon.genres().await.unwrap();
        match genres.as_slice() {
            [] => unreachable!("every webtoons must have a genre"),
            [_genre] => {}
            [_first, rest @ ..] => {
                assert!(rest.is_empty());
            }
        }

        let thumbnail = webtoon.thumbnail().await.unwrap();
        assert!(thumbnail.is_none());

        let banner = webtoon.banner().await.unwrap();
        assert!(banner.is_some());

        let language = webtoon.language();
        assert_eq!(Language::En, language);

        let schedule = webtoon.schedule().await.unwrap();
        assert!(schedule.is_some());

        let _views = webtoon.views().await.unwrap();

        let _subscribers = webtoon.subscribers().await.unwrap();

        let _summary = webtoon.summary().await.unwrap();

        let episode = webtoon.random_episode().await.unwrap();
        eprintln!("episode {}", episode.number());

        let _title = episode.title().await.unwrap();
        let _season = episode.season().await.unwrap();
        let _thumbnail = episode.thumbnail().await.unwrap();
        let _length = episode.length().await.unwrap();
        let _likes = episode.likes().await.unwrap();

        episode
            .posts_for_each(async |post| {
                let _replies = post.replies().await.unwrap();
            })
            .await
            .unwrap();
    }
}
