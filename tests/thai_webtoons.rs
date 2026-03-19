use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    error::EpisodeError,
    meta::Genre,
    originals::{Schedule, Weekday},
};

#[tokio::test]
async fn thai_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals(Language::Th).await.unwrap();
}

#[tokio::test]
async fn thai_canvas_page() {
    let client = Client::new();
    {
        let webtoons = client
            .canvas(Language::Th, 1..2, Sort::Popularity)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client
            .canvas(Language::Th, 1..2, Sort::Likes)
            .await
            .unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(Language::Th, 1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn thai_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url(
            "https://www.webtoons.com/th/fantasy/the-greatest-estate-developer/list?title_no=4646",
        )
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("ยอดสถาปนิกผู้พิทักษ์อาณาจักร", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail);

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20250701_146/1751338524173q59Um_PNG/EpisodeList_PC_Character.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::Th, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_a] => unreachable!("`ยอดสถาปนิกผู้พิทักษ์อาณาจักร` should have more than one creator"),
        [_a, _b, _c, _d, ..] => {
            unreachable!(
                "`ยอดสถาปนิกผู้พิทักษ์อาณาจักร` should have less than four creators: {creators:?}"
            )
        }
        [_a, _b] => {
            unreachable!("`ยอดสถาปนิกผู้พิทักษ์อาณาจักร` should have more than two creators")
        }
        [a, b, c] => {
            assert_eq!(a.username(), "Lee hyunmin");
            assert_eq!(b.username(), "Kim Hyunsoo");
            assert_eq!(c.username(), "BK_Moon");
        }
    }

    let genres = webtoon.genres().await.unwrap();
    match genres.as_slice() {
        [] => unreachable!("every webtoons must have a genre"),
        [_first, _second, _rest @ ..] => {
            unreachable!("Originals can only display one genre on home page")
        }
        [genre] => {
            assert_eq!(Genre::Fantasy, *genre);
        }
    }

    let schedule = webtoon.schedule().await.unwrap().unwrap();
    assert_eq!(Schedule::Weekday(Weekday::Thursday), schedule);

    let views = webtoon.views().await.unwrap();
    assert!(views >= 69_300_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    eprintln!("{subscribers}");
    assert!(subscribers >= 448_591);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "นักศึกษาวิศวกรรมโยธา คิมซูโฮ ผล็อยหลับระหว่างอ่านนิยายแฟนตาซี เมื่อตื่นขึ้นมาก็กลายมาเป็นตัวละครในนั้นซะแล้ว! เจ้าของร่างของเขามีชื่อว่า ‘ลอยด์ ฟรอนเทร่า’ หนุ่มชนชั้นสูงผู้ขี้เกียจตัวเป็นขน แล้วยังชอบเมาหัวราน้ำเป็นชีวิตจิตใจ มิหนำซ้ำครอบครัวของเขายังเป็นหนี้กองโตเท่าภูเขาอีก ซูโฮจึงประยุกต์ใช้ความรู้ทางด้านวิศวกรรมที่มีอยู่ เพื่อเริ่มต้นชีวิตใหม่ (?) อย่างใสสะอาด พร้อมความช่วยเหลือจากเจ้าแฮมสเตอร์ยักษ์ อัศวิน และเวทมนตร์ของโลกใบนี้!",
        summary
    );

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(169000, length);

    let likes = episode.likes().await.unwrap();
    assert!(likes >= 27_418, "likes were {likes}");

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    if let Some(_post) = posts.next().await.unwrap() {}
}

#[tokio::test]
async fn thai_original_can_have_empty_summary() {
    let client = Client::new();
    let webtoon = client.webtoon(5709, Type::Original).await.unwrap().unwrap();

    assert_eq!(Language::Th, webtoon.language());
    assert!(webtoon.summary().await.unwrap().is_empty());
}

#[tokio::test]
async fn thai_webtoon_original_episode_can_only_be_read_on_epp() {
    let client = Client::new();

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/th/fantasy/uq-holder/list?title_no=5617")
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("UQ HOLDER! [รายเล่ม]", title);

    let language = webtoon.language();
    assert_eq!(Language::Th, language);

    let schedule = webtoon.schedule().await.unwrap().unwrap();
    assert_eq!(Schedule::Completed, schedule);

    let episode = webtoon.episode(1).await.unwrap().unwrap();

    match episode.title().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    match episode.season().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    match episode.thumbnail().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    match episode.panels().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    match episode.note().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    match episode.length().await {
        Err(EpisodeError::NotViewable) => {}
        _ => unreachable!("epsiode should only be viewable on the app, and this is `NotViewable`"),
    }

    let likes = episode.likes().await.unwrap();
    assert!(likes >= 164, "likes were {likes}");

    let (_comments, _replies) = episode.comments_and_replies().await.unwrap();

    let mut posts = episode.posts();
    if let Some(_post) = posts.next().await.unwrap() {}
}
