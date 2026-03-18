use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    meta::Genre,
    originals::{Schedule, Weekday},
};

#[tokio::test]
async fn indonesian_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals(Language::Id).await.unwrap();
}

#[tokio::test]
async fn indonesian_canvas_page() {
    let client = Client::new();
    {
        let webtoons = client
            .canvas(Language::Id, 1..2, Sort::Popularity)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client
            .canvas(Language::Id, 1..2, Sort::Likes)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(Language::Id, 1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn indonesian_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/id/romance/iseops-romance/list?title_no=5660")
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("Iseop's Romance", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail);

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20260116_119/1768557364567uQjHP_PNG/Iseop's%20Romance_EpisodeList_PC_Character.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::Id, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_a] => unreachable!("`Iseop's Romance` should have more than one creator"),
        [_a, _b, _c, _d, ..] => {
            unreachable!("`Iseop's Romance` should have less than four creators: {creators:?}")
        }
        [_a, _b, _c] => {
            unreachable!("`Iseop's Romance` should have less than three creators")
        }
        [a, b] => {
            assert_eq!(a.username(), "248");
            assert_eq!(b.username(), "Anna Kim");
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
    assert_eq!(Schedule::Weekday(Weekday::Tuesday), schedule);

    let views = webtoon.views().await.unwrap();
    assert!(views >= 84_000_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    eprintln!("{subscribers}");
    assert!(subscribers >= 1_300_000);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "Kang Minkyung adalah karyawan terbaik di TK grup yang karirnya melejit dalam waktu singkat. Namun pada suatu hari, tiba-tiba ia ditunjuk menjadi sekretaris sang direktur eksekutif, Tae Iseop! Bagaimana kisah cinta kantoran antara seorang anak konglomerat yang 'mageran' dan sekretarisnya yang kelewat ambis?!",
        summary
    );

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(51517, length);

    let likes = episode.likes().await.unwrap();
    assert!(likes >= 119_994, "likes were {likes}");

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    if let Some(_post) = posts.next().await.unwrap() {}
}

#[tokio::test]
async fn indonesian_genre_kriminal_misteri_is_mystery() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1127801, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Id, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [horror, mystery] => {
            assert_eq!(Genre::Horror, *horror);
            assert_eq!(Genre::Mystery, *mystery);
        }

        _ => unreachable!("`Antenne Râteau` should have two genre"),
    }
}

#[tokio::test]
async fn indonesian_schedule_everyday() {
    let client = Client::new();
    let webtoon = client.webtoon(9776, Type::Original).await.unwrap().unwrap();

    assert_eq!(Language::Id, webtoon.language());

    let schedule = webtoon.schedule().await.unwrap().unwrap();
    assert_eq!(Schedule::Daily, schedule);
}

#[tokio::test]
async fn indonesian_canvas_creator_does_not_have_a_profile() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1066566, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::Id, webtoon.language());

    let creator = webtoon.creators().await.unwrap();

    match creator.as_slice() {
        [a] => {
            assert_eq!("Pluto's_alien", a.username());
            assert!(a.profile().is_none());
        }

        _ => unreachable!("Canvas stories should only have one creator, got: {creator:?}"),
    }
}
