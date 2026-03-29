#![allow(clippy::ignore_without_reason)]
use webtoon::platform::webtoons::{Client, Language, canvas::Sort, error::EpisodeError};

#[tokio::test]
#[ignore]
async fn canvas() {
    let client = Client::new();

    let page = fastrand::u16(1..=300);

    let canvas = client
        .canvas(Language::Th, page..=page, Sort::Popularity)
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
                _ = creator.id();
                _ = creator.followers();
                _ = creator.has_patreon();
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
        assert_eq!(Language::Th, language);

        let schedule = webtoon.schedule().await.unwrap();
        assert!(schedule.is_none());

        let _views = webtoon.views().await.unwrap();

        let _subscribers = webtoon.subscribers().await.unwrap();

        let _summary = webtoon.summary().await.unwrap();

        let episode = webtoon.random_episode().await.unwrap();
        eprintln!("episode {}", episode.number());

        // TODO: use `assert_matches!` when stabilized.
        let title = episode.title().await;
        assert!(
            matches!(title, Ok(_) | Err(EpisodeError::NotViewable)),
            "{title:?}"
        );

        let season = episode.season().await;
        assert!(
            matches!(season, Ok(_) | Err(EpisodeError::NotViewable)),
            "{season:?}"
        );
        let thumbnail = episode.thumbnail().await;
        assert!(
            matches!(thumbnail, Ok(_) | Err(EpisodeError::NotViewable)),
            "{thumbnail:?}"
        );
        let length = episode.length().await;
        assert!(
            matches!(length, Ok(_) | Err(EpisodeError::NotViewable)),
            "{length:?}"
        );

        let _likes = episode.likes().await.unwrap();

        let mut comments = episode.posts();

        while let Some(comment) = comments.next().await.unwrap() {
            let _replies = comment.replies().await.unwrap();
        }
    }
}

#[tokio::test]
#[ignore]
async fn originals() {
    let client = Client::new();

    let originals = client.originals(Language::Th).await.unwrap();

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
                    _ = creator.id();
                    _ = creator.followers();
                    _ = creator.has_patreon();
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
        assert_eq!(Language::Th, language);

        let schedule = webtoon.schedule().await.unwrap();
        assert!(schedule.is_some());

        let _views = webtoon.views().await.unwrap();

        let _subscribers = webtoon.subscribers().await.unwrap();

        let _summary = webtoon.summary().await.unwrap();

        let episode = webtoon.random_episode().await.unwrap();
        eprintln!("episode {}", episode.number());

        let title = episode.title().await;
        assert!(
            matches!(title, Ok(_) | Err(EpisodeError::NotViewable)),
            "{title:?}"
        );

        let season = episode.season().await;
        assert!(
            matches!(season, Ok(_) | Err(EpisodeError::NotViewable)),
            "{season:?}"
        );
        let thumbnail = episode.thumbnail().await;
        assert!(
            matches!(thumbnail, Ok(_) | Err(EpisodeError::NotViewable)),
            "{thumbnail:?}"
        );
        let length = episode.length().await;
        assert!(
            matches!(length, Ok(_) | Err(EpisodeError::NotViewable)),
            "{length:?}"
        );

        let _likes = episode.likes().await.unwrap();

        let mut comments = episode.posts();
        while let Some(comment) = comments.next().await.unwrap() {
            let _replies = comment.replies().await.unwrap();
        }
    }
}
