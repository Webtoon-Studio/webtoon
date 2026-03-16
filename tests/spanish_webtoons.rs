use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    meta::Genre,
    originals::{Schedule, Weekday},
};

#[tokio::test]
async fn spanish_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals(Language::Es).await.unwrap();
}

#[tokio::test]
async fn spanish_canvas_page() {
    let client = Client::new();
    {
        let webtoons = client
            .canvas(Language::Es, 1..2, Sort::Popularity)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client
            .canvas(Language::Es, 1..2, Sort::Likes)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(Language::Es, 1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn spanish_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url(
            "https://www.webtoons.com/es/romance/who-stole-the-empress/list?title_no=6477",
        )
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("¿Quién se robó a la emperatriz?", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail);

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20231004_92/16963836731854XwTC_PNG/1Who-Stole-the-Empress_landingpage_desktop_character.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::Es, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_a] => unreachable!("`¿Quién se robó a la emperatriz?` should have more than one creator"),
        [_a, _b] => {
            unreachable!("`¿Quién se robó a la emperatriz?` should have more than two creators")
        }
        [_a, _b, _c, _d, ..] => {
            unreachable!(
                "`¿Quién se robó a la emperatriz?` should have less than four creators: {creators:?}"
            )
        }
        [a, b, c] => {
            assert_eq!(a.username(), "Lee jihye");
            assert_eq!(b.username(), "Muhly");
            assert_eq!(c.username(), "Pinku");
        }
    }

    let genres = webtoon.genres().await.unwrap();
    match genres.as_slice() {
        [] => unreachable!("every webtoons must have a genre"),
        [_first, _second, _rest @ ..] => {
            unreachable!("Originals can only display one genre on home page")
        }
        [genre] => {
            assert_eq!(Genre::Romance, *genre);
        }
    }

    let schedule = webtoon.schedule().await.unwrap().unwrap();
    assert_eq!(Schedule::Weekday(Weekday::Friday), schedule);

    let views = webtoon.views().await.unwrap();
    assert!(views >= 9_100_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    eprintln!("{subscribers}");
    assert!(subscribers >= 249_090);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "La emperatriz Roselyn V. Sunsett, quien fue acusada de una traición que no cometió, es abandonada a su suerte. Su imperio y sus seres queridos caen, y ella, desolada, quiere morir. Sin embargo, alguien de una nación enemiga la encuentra y se la lleva a su imperio, con el fin de usarla para vengarse de sus enemigos.",
        summary
    );

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(239026, length);

    let likes = episode.likes().await.unwrap();
    assert!(likes >= 31079, "likes were {likes}");

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    if let Some(_post) = posts.next().await.unwrap() {}
}

#[tokio::test]
async fn spanish_genre_historia_corta_is_short_story() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1128134, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [short_story, scifi] => {
            assert_eq!(Genre::ShortStory, *short_story);
            assert_eq!(Genre::SciFi, *scifi);
        }

        _ => unreachable!("`Fábrica de monos` should have two generes"),
    }
}

#[tokio::test]
async fn spanish_genre_crimen_misterio_is_mystery() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1127650, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [mystery, romance] => {
            assert_eq!(Genre::Mystery, *mystery);
            assert_eq!(Genre::Romance, *romance);
        }

        _ => unreachable!("`Mi Vida Sensible` should have two generes"),
    }
}

#[tokio::test]
async fn spanish_genre_fantasía_romántica_is_romantic_fantasy() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1123874, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [romantic_fantasy, lgbtq] => {
            assert_eq!(Genre::RomanticFantasy, *romantic_fantasy);
            assert_eq!(Genre::LGBTQ, *lgbtq);
        }

        _ => unreachable!("`Cazadores` should have two generes"),
    }
}

#[tokio::test]
async fn spanish_genre_postapocalíptico_is_post_apocalyptic() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1114348, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [scifi, post_apocalyptic] => {
            assert_eq!(Genre::SciFi, *scifi);
            assert_eq!(Genre::PostApocalyptic, *post_apocalyptic);
        }

        _ => unreachable!("`CACERIA CIBERNETICA 01` should have two generes"),
    }
}

#[tokio::test]
async fn spanish_genre_misterio_is_mystery() {
    let client = Client::new();
    let webtoon = client.webtoon(8433, Type::Original).await.unwrap().unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [mystery] => {
            assert_eq!(Genre::Mystery, *mystery);
        }

        _ => unreachable!("`Señorita Cometa` should have one genre"),
    }
}

#[tokio::test]
async fn spanish_genre_inspirador_is_inspirational() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1123106, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Es, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [short_story, inspirational] => {
            assert_eq!(Genre::ShortStory, *short_story);
            assert_eq!(Genre::Inspirational, *inspirational);
        }

        _ => unreachable!("`Imay - ¿Quien Soy?` should have two genres"),
    }
}
