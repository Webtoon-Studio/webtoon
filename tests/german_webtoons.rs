use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    meta::Genre,
    originals::{Schedule, Weekday},
};

#[tokio::test]
async fn german_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals(Language::De).await.unwrap();
}

#[tokio::test]
async fn german_canvas_page() {
    let client = Client::new();
    {
        let webtoons = client
            .canvas(Language::De, 1..2, Sort::Popularity)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client
            .canvas(Language::De, 1..2, Sort::Likes)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(Language::De, 1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn german_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/de/action/nano-machine/list?title_no=4511")
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("Nano-Maschine", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail.as_deref());

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20230510_283/1683700626305rnAIp_PNG/410_Detail_PC_cha__1_.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::De, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_a] => unreachable!("`Nano-Maschine` should have more than one creator"),
        [_a, _b] => unreachable!("`Nano-Maschine` should have more than two creators"),
        [_a, _b, _c, _d, ..] => {
            unreachable!("`Nano-Maschine` should have less than four creators: {creators:?}")
        }
        [a, b, c] => {
            assert_eq!(a.username(), "Great H");
            assert_eq!(b.username(), "GGBG");
            assert_eq!(c.username(), "HANJUNG WOLYA");
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
    assert!(views >= 2_600_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    assert!(subscribers >= 17_554);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "Nanotechnologie trifft auf Martial Arts! Yeoun Cheons Mutter gehört zwar nicht zu den sechs offiziellen Ehefrauen des Anführers der Dämonensekte, aber seine Herkunft qualifiziert ihn dennoch dazu, sein Können in den Aufnahmeprüfungen der Dämonenakademie unter Beweis zu stellen. Wird die geheimnisvolle Nanotechnologie, die er von einem Nachfahren aus der Zukunft erhalten hat, ausreichen, um den harten Wettbewerb gegen seine mächtigen Halbgeschwister zu bestehen?",
        summary
    );

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(130054, length);

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    while let Some(post) = posts.next().await.unwrap() {
        eprintln!("{}", post.body().contents());
    }

    if let Some(episode) = webtoon.episodes().await.unwrap().into_iter().next() {
        let published = episode.published().unwrap();
        assert_eq!(4, published.day());
        assert_eq!(7, published.month());
        assert_eq!(2022, published.year());
    }
}

#[tokio::test]
async fn german_originals_romantasy_is_romantic_fantasy() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1119143, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(Language::De, webtoon.language());

    match webtoon.genres().await.unwrap().as_slice() {
        [fantasy, romantasy] => {
            assert_eq!(Genre::Fantasy, *fantasy);
            assert_eq!(Genre::RomanticFantasy, *romantasy);
        }

        _ => unreachable!("`The Dragon Legion` should have two generes"),
    }
}
