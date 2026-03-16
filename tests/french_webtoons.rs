use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    meta::Genre,
    originals::{Schedule, Weekday},
};

#[tokio::test]
async fn french_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals(Language::Fr).await.unwrap();
}

#[tokio::test]
async fn french_canvas_page() {
    let client = Client::new();
    {
        let webtoons = client
            .canvas(Language::Fr, 1..2, Sort::Popularity)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client
            .canvas(Language::Fr, 1..2, Sort::Likes)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(Language::Fr, 1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn french_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url(
            "https://www.webtoons.com/fr/action/absolute-regression/list?title_no=7341",
        )
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("Absolute Regression", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail);

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20241031_17/1730316261548G9dz7_PNG/1EpisodeList_PC_Character.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::Fr, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_a] => unreachable!("`Absolute Regression` should have more than one creator"),
        [_a, _b] => {
            unreachable!("`Absolute Regression` should have more than two creators")
        }
        [_a, _b, _c, _d, ..] => {
            unreachable!("`Absolute Regression` should have less than four creators: {creators:?}")
        }
        [a, b, c] => {
            assert_eq!(a.username(), "Y.H. JANG");
            assert_eq!(b.username(), "JP");
            assert_eq!(c.username(), "PARK JIN HWAN");
        }
    }

    let genres = webtoon.genres().await.unwrap();
    match genres.as_slice() {
        [] => unreachable!("every webtoons must have a genre"),
        [_first, _second, _rest @ ..] => {
            unreachable!("Originals can only display one genre on home page")
        }
        [genre] => {
            assert_eq!(Genre::Action, *genre);
        }
    }

    let schedule = webtoon.schedule().await.unwrap().unwrap();
    assert_eq!(Schedule::Weekday(Weekday::Monday), schedule);

    let views = webtoon.views().await.unwrap();
    assert!(views >= 3_600_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    eprintln!("{subscribers}");
    assert!(subscribers >= 52_782);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "Le jeune maître du culte démoniaque a perdu tous les siens. Incapable de se venger du responsable, il dédie sa vie entière à réunir les ingrédients nécessaires pour retourner dans le passé. Maintenant qu'il est de retour, il va tenter de résoudre tous ses regrets dans cette nouvelle vie.",
        summary
    );

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(295213, length);

    let likes = episode.likes().await.unwrap();
    assert!(likes >= 4_272, "likes were {likes}");

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    if let Some(_post) = posts.next().await.unwrap() {}
}

#[tokio::test]
async fn french_genre_éducatif_is_informative() {
    let client = Client::new();
    let webtoon = client.webtoon(7241, Type::Original).await.unwrap().unwrap();

    assert_eq!(Language::Fr, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [informative] => {
            assert_eq!(Genre::Informative, *informative);
        }

        _ => unreachable!("`5 ans de WEBTOON !` should have one genre"),
    }
}
