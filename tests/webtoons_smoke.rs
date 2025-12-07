use webtoon::platform::webtoons::{Client, Language, canvas::Sort, webtoon::post::Posts};

#[tokio::test]
#[ignore]
async fn smoke() {
    let client = Client::new();

    let page = fastrand::u16(1..=1000);

    let canvas = client
        .canvas(Language::En, page..=page, Sort::Popularity)
        .await
        .unwrap_or_else(|err| panic!("failed on page {page}: {err}"));

    assert_eq!(20, canvas.len());

    for webtoon in canvas {
        eprintln!("\nChecking webtoon {}", webtoon.id());

        let _title = webtoon.title().await.unwrap();

        let creators = webtoon.creators().await.unwrap();
        match creators.as_slice() {
            [] => unreachable!("every webtoon must have a creator"),
            [_first, _second, _rest @ ..] => {
                unreachable!("`canvas stories can only have one creator")
            }
            [creator] => {
                let _has_patreon = creator.has_patreon().await.unwrap();
                let _id = creator.id().await.unwrap();
            }
        }

        let genres = webtoon.genres().await.unwrap();
        match genres.as_slice() {
            [] => unreachable!("every webtoons must have a genre"),
            [_genre] => {}
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

        let _likes = webtoon.likes().await.unwrap();

        let _subscribers = webtoon.subscribers().await.unwrap();

        let _summary = webtoon.summary().await.unwrap();

        // TODO: Add a `random_episode` (or some way to get a random episode)
        // so that tests extend beyond just the first episode.
        //
        // Also need to handle an episode not being viewable. Currently it just
        // panics.
        let episode = webtoon.first_episode().await.unwrap();
        eprintln!("episode {}", episode.number());

        let _title = episode.title().await.unwrap();
        let _season = episode.season().await.unwrap();
        let _thumbnail = episode.thumbnail().await.unwrap();
        let _length = episode.length().await.unwrap();

        episode
            .posts_for_each(async |post| {
                eprintln!("Checking post: {}", post.id());
                let _replies = post.replies::<u32>().await.unwrap();
                let _replies = post.replies::<Posts>().await.unwrap();
            })
            .await
            .unwrap();
    }
}
