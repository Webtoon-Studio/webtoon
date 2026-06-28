use std::assert_matches;

use webtoon::platform::webtoons::{
    Client, Type,
    canvas::Sort,
    error::CreatorError,
    originals::{Schedule, Weekday},
    webtoon::Genre,
};

#[tokio::test]
async fn user_info_should_deserialize_even_with_invalid_session() {
    let client = Client::new();

    let user_info = client
        .user_info_for_session("not-a-real-session")
        .await
        .unwrap();

    assert!(user_info.is_none());
}

#[tokio::test]
async fn english_search() {
    let client = Client::new();
    let _search = client.search("Universe").await.unwrap();
}

#[tokio::test]
async fn english_creator() {
    let client = Client::new();

    let creator = client.creator("JennyToons").await.unwrap().unwrap();

    let username = creator.username();
    assert_eq!("Jenny-Toons", username);

    let profile = creator.profile();
    assert_eq!(Some("JennyToons"), profile);

    let id = creator.id().await.unwrap();
    assert_eq!(Some("n5z4d"), id.as_deref());

    let followers = creator.followers().await.unwrap();
    assert!(followers.is_some());

    let has_patreon = creator.has_patreon().await.unwrap();
    assert_eq!(Some(true), has_patreon);

    let webtoons = creator.webtoons().await.unwrap().unwrap();
    match webtoons.as_slice() {
        [_one, _two, _three, _four, _five] => {}
        _ => unreachable!("Jenny has 5 webtoons"),
    }
}

#[tokio::test]
async fn english_originals_page() {
    let client = Client::new();
    let _webtoons = client.originals().await.unwrap();
}

#[tokio::test]
async fn english_canvas_page() {
    let client = Client::new();

    {
        let webtoons = client.canvas(1..2, Sort::Popularity).await.unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(1..2, Sort::Likes).await.unwrap();
        for _webtoon in webtoons {}
    }

    {
        let webtoons = client.canvas(1..2, Sort::Date).await.unwrap();
        for _webtoon in webtoons {}
    }
}

#[tokio::test]
async fn english_webtoon_canvas() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/en/canvas/testing-service/list?title_no=843910")
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("Testing Service", title);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("`canvas stories can only have one creator")
        }
        [creator] => {
            assert_eq!("RoloEdits", creator.username());
            assert_eq!(Some("_nb3nw"), creator.profile());
            assert!(!creator.has_patreon().await.unwrap().unwrap());
        }
    }

    let genres = webtoon.genres().await.unwrap();
    match genres.as_slice() {
        [] => unreachable!("every webtoons must have a genre"),
        [_genre] => {
            unreachable!("`Testing Service` should have more than one genre");
        }
        [first, second, rest @ ..] => {
            assert_eq!(Genre::SciFi, *first);
            assert_eq!(Genre::Drama, *second);
            assert!(rest.is_empty());
        }
    }

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20230205_197/1675547968753uULHz_PNG/7f35dc9f-5307-4595-bcef-350be53ce2338623572264572340167.png"
        ),
        thumbnail.as_deref()
    );

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(None, banner);

    let schedule = webtoon.schedule().await.unwrap();
    assert!(schedule.is_none());

    let _views = webtoon.views().await.unwrap();

    let _likes = webtoon.likes().await.unwrap();

    let _subscribers = webtoon.subscribers().await.unwrap();

    let summary = webtoon.summary().await.unwrap();
    assert_eq!("test", summary);
}

#[tokio::test]
async fn english_webtoon_original() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/en/romance/i-am-the-villain/list?title_no=4937")
        .unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("I Am the Villain", title);

    let thumbnail = webtoon.thumbnail().await.unwrap();
    assert_eq!(None, thumbnail.as_deref());

    let banner = webtoon.banner().await.unwrap();
    assert_eq!(
        Some(
            "https://swebtoon-phinf.pstatic.net/20260128_210/1769549433316rbQCg_PNG/4I_Am_the_Villain_Landing_Page_PC_Character.png"
        ),
        banner.as_deref()
    );

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("`I Am the Villain` should only have one creator")
        }
        [creator] => {
            assert_eq!("Sejji", creator.username());
            assert_eq!(Some("08x59"), creator.profile());
            assert_eq!(Some(true), creator.has_patreon().await.unwrap());
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
    assert_eq!(Schedule::Weekday(Weekday::Thursday), schedule);

    let views = webtoon.views().await.unwrap();
    assert!(views >= 64_300_000);

    let subscribers = webtoon.subscribers().await.unwrap();
    assert!(subscribers >= 1_200_000);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "Working hard is supposed to take you far - but into the world of your best friend's novel? For Lucy, being whisked into a life of ballrooms and picnics isn't exactly what it's cracked up to be; at least, not when everyone has mistaken her for the villain flagged for death. To escape this fate, Lucy must transform from modern workaholic to high society schemer if she even has a chance at returning home. Will she make it? Or will this world of beautiful outfits, strawberry desserts, and dashingly handsome gentlemen seal her fate?",
        summary
    );

    // PERF: `webtoon.likes()` fetches every episode and then sums up the likes
    // for each. This can be very slow, so we just try one episode and get its
    // likes. This can mean that not everything will be caught, but it speeds up
    // the test dramatically.
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    let likes = episode.likes().await.unwrap();
    assert!(likes >= 134_305, "likes were {likes}");
}

#[tokio::test]
async fn english_original_episode_with_normal_reader() {
    let client = Client::new();

    let webtoon = client.webtoon(7492, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(19)
        .await
        .unwrap()
        .expect("No episode for given number");

    assert_eq!("Episode 19", episode.title().await.unwrap());
    assert_eq!(Some(111800), episode.length().await.unwrap());
}

#[tokio::test]
async fn english_canvas_episode_with_normal_reader() {
    let client = Client::new();

    let webtoon = client
        .webtoon(1082723, Type::Canvas)
        .await
        .unwrap()
        .unwrap();

    let episode = webtoon
        .episode(10)
        .await
        .unwrap()
        .expect("No episode for given number");

    assert_eq!("EPISODE 2 PART 3", episode.title().await.unwrap());
    assert_eq!(Some(26860), episode.length().await.unwrap());
}

#[tokio::test]
async fn english_original_episode_with_alternate_reader() {
    let client = Client::new();

    let webtoon = client.webtoon(4784, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(1)
        .await
        .unwrap()
        .expect("No episode for given number");

    assert_eq!(
        "Ep. 1 - The Busan Karaoke Ghost",
        episode.title().await.unwrap()
    );
    assert_eq!(None, episode.length().await.unwrap());
}

#[tokio::test]
async fn english_canvas_posts() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();

    let episode = webtoon
        .episode(2)
        .await
        .unwrap()
        .expect("No episode for given number");

    let mut comments = episode.posts();

    while let Some(comment) = comments.next().await.unwrap() {
        for reply in comment.replies().await.unwrap() {
            _ = std::hint::black_box(reply);
        }
    }
}

#[tokio::test]
async fn english_original_posts() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client.webtoon(505, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(50)
        .await
        .unwrap()
        .expect("No episode for given number");

    let mut comments = episode.posts();

    while let Some(comment) = comments.next().await.unwrap() {
        for _reply in comment.replies().await.unwrap() {}
    }
}

#[tokio::test]
async fn english_canvas_download_single() {
    let client = Client::new();

    let webtoon = client.webtoon(693372, Type::Canvas).await.unwrap().unwrap();

    let episode = webtoon
        .episode(219)
        .await
        .unwrap()
        .expect("No episode for given number");

    let panels = episode.download().await.unwrap();

    assert_eq!(15, panels.count());

    // Save as a single, long image.
    panels
        .save_single("target/tmp/english_canvas_single/")
        .await
        .unwrap();
}

#[tokio::test]
async fn english_original_download_single() {
    let client = Client::new();

    let webtoon = client.webtoon(1099, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(1)
        .await
        .unwrap()
        .expect("No episode for given number");

    let panels = episode.download().await.unwrap();

    assert_eq!(43, panels.count());

    // Save as a single, long image.
    panels
        .save_single("target/tmp/english_original_single/")
        .await
        .unwrap();
}

#[tokio::test]
async fn english_canvas_download_multi() {
    let client = Client::new();

    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();

    let episode = webtoon
        .episode(1)
        .await
        .unwrap()
        .expect("No episode for given number");

    let panels = episode.download().await.unwrap();

    assert_eq!(1, panels.count());

    // Save each individual panel as a separate image.
    panels
        .save_multiple("target/tmp/english_canvas_multi/")
        .await
        .unwrap();
}

#[tokio::test]
async fn english_original_download_multi() {
    let client = Client::new();

    let webtoon = client.webtoon(5896, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(1)
        .await
        .unwrap()
        .expect("No episode for given number");

    let panels = episode.download().await.unwrap();

    assert_eq!(160, panels.count());

    // Save each individual panel as a separate image.
    panels
        .save_multiple("target/tmp/english_original_multi/")
        .await
        .unwrap();
}

#[tokio::test]
async fn english_original_rss() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let rss = webtoon.rss().await.unwrap();

    assert_eq!("Tower of God", rss.title());
}

#[tokio::test]
async fn english_canvas_rss() {
    let client = Client::new();
    let webtoon = client.webtoon(135963, Type::Canvas).await.unwrap().unwrap();
    let rss = webtoon.rss().await.unwrap();

    assert_eq!("Nerd and Jock", rss.title());
}

#[tokio::test]
async fn english_canvas_panel_pixels_non_zero_decimal() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1085313, Type::Canvas)
        .await
        .unwrap()
        .unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();

    assert_eq!(37216, length);
}

#[tokio::test]
async fn english_canvas_panel_pixels_height_more_than_1280() {
    let client = Client::new();
    let webtoon = client.webtoon(903679, Type::Canvas).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let length = episode.length().await.unwrap().unwrap();

    assert_eq!(3384, length);
}

#[tokio::test]
#[allow(nonstandard_style, reason = "test is checking for `JPEG` vs `jpeg`")]
async fn english_canvas_panel_JPEG_ext() {
    let client = Client::new();
    let webtoon = client
        .webtoon(1081912, Type::Canvas)
        .await
        .unwrap()
        .unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();

    let title = episode.title().await.unwrap();

    assert_eq!("Episode 1", title);
}

#[tokio::test]
async fn english_canvas_comma_in_username_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(738855, Type::Canvas).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("canvas stories can only have one creator: {creators:#?}")
        }
        [_creator] => {}
    }
}

#[tokio::test]
async fn english_canvas_space_in_username_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(910844, Type::Canvas).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("canvas stories can only have one creator: {creators:#?}")
        }
        [_creator] => {}
    }
}

#[tokio::test]
async fn english_originals_space_in_username_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(1499, Type::Original).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("this original should can only have one creator: {creators:#?}")
        }
        [_creator] => {}
    }
}

#[tokio::test]
async fn english_originals_multi_korean_creators_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(2135, Type::Original).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [first, second, third] => {
            assert_eq!("SUMPUL", first.username());
            assert_eq!("HereLee", second.username());
            assert_eq!("Alphatart", third.username());
        }
        _ => unreachable!("this webtoon has three creators: {creators:#?}"),
    }
}

#[tokio::test]
async fn english_originals_multi_english_creators_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(1881, Type::Original).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [first, second] => {
            assert_eq!("Anne Delseit", first.username());
            assert_eq!("Marissa Delbressine", second.username());
        }
        _ => unreachable!("this webtoon has two creators: {creators:#?}"),
    }
}

#[tokio::test]
async fn english_originals_korean_creator_with_spaces_should_be_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(6670, Type::Original).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [creator] => {
            assert_eq!("kang eun young", creator.username());
        }
        _ => unreachable!("this webtoon has one creators: {creators:#?}"),
    }
}

#[tokio::test]
async fn english_originals_with_gif() {
    let client = Client::new();
    let webtoon = client.webtoon(2757, Type::Original).await.unwrap().unwrap();

    let episode = webtoon.episode(25).await.unwrap().unwrap();

    let panels = episode.download().await.unwrap();
    panels
        .save_single("target/tmp/gif_in_single_png/")
        .await
        .unwrap();
}

#[tokio::test]
async fn english_canvas_creator_with_multiple_trailing_dots_is_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(557095, Type::Canvas).await.unwrap().unwrap();
    // `That 1 kid.....`
    let creators = webtoon.creators().await.unwrap();
    assert!(creators.len() == 1);
}

#[tokio::test]
async fn english_canvas_panel_image_with_multiple_dots_in_ext_is_ok() {
    let client = Client::new();
    let webtoon = client.webtoon(460550, Type::Canvas).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    // For when images ended with: `1.7.jpeg`. Make sure to only get the `jpeg` part.
    let length = episode.length().await.unwrap().unwrap();
    assert_eq!(7715, length);
}

#[tokio::test]
async fn english_canvas_creator_invalid_creator_profile() {
    let client = Client::new();
    let creator = client.creator("y87lz").await;

    match creator {
        Err(CreatorError::InvalidCreatorProfile) => {}
        Ok(_) | Err(_) => unreachable!("should return `InvalidCreatorProfile` error: {creator:#?}"),
    }
}

#[tokio::test]
async fn english_original_creator_of_korean_story_should_scrape_fine() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();

    let title = webtoon.title().await.unwrap();
    assert_eq!("Tower of God", title);

    let creators = webtoon.creators().await.unwrap();

    if let [creator] = creators.as_slice() {
        assert_eq!("SIU", creator.username());
        assert!(creator.profile().is_none());
        assert!(creator.id().await.unwrap().is_none());
        assert!(creator.followers().await.unwrap().is_none());
        assert!(creator.has_patreon().await.unwrap().is_none());
    } else {
        unreachable!("should find SIU on Tower of God: {creators:?}");
    }
}

#[tokio::test]
async fn english_canvas_creator_should_not_have_html_encoded_text() {
    let client = Client::new();
    let webtoon = client.webtoon(808662, Type::Canvas).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();

    if let [creator] = creators.as_slice() {
        assert_eq!("Ash xx<33", creator.username());
    } else {
        unreachable!("should find SIU on Tower of God: {creators:?}");
    }
}

#[tokio::test]
async fn english_canvas_creator_names_can_have_spaces_at_end() {
    let client = Client::new();
    let webtoon = client.webtoon(796674, Type::Canvas).await.unwrap().unwrap();

    let creators = webtoon.creators().await.unwrap();

    if let [creator] = creators.as_slice() {
        // FIX: This should have a space at the end, but due to the cleaning
        // of the creators names (because `webtoons.com` makes a mess of the names)
        // we just trim.
        //
        // This is not correct! The names should be maintained, but there isn't
        // really a good way to achieve this for now.
        assert_eq!("illustraboxstudios", creator.username());
    } else {
        unreachable!("should find creator: {creators:?}");
    }
}

#[tokio::test]
async fn english_webtoon_genre() {
    let client = Client::new();

    // Romance Fantasy
    {
        let webtoon = client
            .webtoon_from_url(
                "https://www.webtoons.com/en/canvas/the-phoenix-the-fearless/list?title_no=1113767",
            )
            .unwrap();

        let genres = webtoon.genres().await.unwrap();
        match genres.as_slice() {
            [] => unreachable!("every webtoons must have a genre"),
            [genre] => {
                assert_eq!(Genre::RomanticFantasy, *genre);
            }
            _ => unreachable!("'The Phoenix & the Fearless' should only have one genre"),
        }
    }
}

#[tokio::test]
async fn english_canvas_invalid_creator_profile() {
    let client = Client::new();

    for profile in ["_91ms9c", "y87lz", "k7yid", "m8sw0"] {
        match client.creator(profile).await {
            Err(CreatorError::InvalidCreatorProfile) => {}
            creator => {
                unreachable!("should return `InvalidCreatorProfile` error: {creator:#?}")
            }
        }
    }
}

#[tokio::test]
async fn english_canvas_creator_page_is_disabled_for_community_policy_violation() {
    let client = Client::new();

    for profile in ["_dcrhv7", "_pdi0q8", "_o2pgx6"] {
        match client.creator(profile).await {
            Ok(None) => {}
            _ => unreachable!("Creator profile page should be disabled for community violations"),
        }
    }

    // Sanity check for getting the creator page from the webtoon page.

    let webtoon = client.webtoon(939253, Type::Canvas).await.unwrap().unwrap();

    match webtoon.creators().await.unwrap().as_slice() {
        [creator] => {
            assert_eq!("Baby Liska", creator.username());
            assert_eq!(Some("_o2pgx6"), creator.profile());
            assert_eq!(None, creator.id().await.unwrap());
            assert_eq!(None, creator.followers().await.unwrap());
            assert_eq!(None, creator.has_patreon().await.unwrap());
            assert_eq!(None, creator.webtoons().await.unwrap());
        }
        creators => unreachable!("should find creator: {creators:?}"),
    }
}

// TODO: Story completed, need to find a new one.
// #[tokio::test]
// async fn english_webtoon_everyday_is_daily_schedule() {
//     let client = Client::new();

//     let webtoon = client
//         .webtoon_from_url(
//             "https://www.webtoons.com/en/romance/goodbye-my-juliet/list?title_no=9870",
//         )
//         .unwrap();

//     match webtoon.schedule().await.unwrap() {
//         Some(Schedule::Daily) => {}
//         _ => unreachable!(),
//     }
// }

#[tokio::test]
async fn only_english_webtoons_com_supported() {
    use webtoon::platform::webtoons::error::{ClientError, InvalidWebtoonUrl};

    let client = Client::new();

    let webtoon = client.webtoon(7418, Type::Original).await;
    assert_matches!(webtoon, Err(ClientError::UnsupportedLanguage));

    let webtoon = client
        .webtoon_from_url("https://www.webtoons.com/de/drama/high-society/list?title_no=7418");
    assert_matches!(webtoon, Err(InvalidWebtoonUrl::UnsupportedLanguage));
}

#[tokio::test]
async fn webtoon_from_url_errors_on_malformed_url() {
    let client = Client::new();
    assert!(client.webtoon_from_url("not-a-url").is_err());
    // missing title_no
    assert!(
        client
            .webtoon_from_url("https://www.webtoons.com/en/fantasy/tower-of-god/list")
            .is_err()
    );
    // empty title_no
    assert!(
        client
            .webtoon_from_url("https://www.webtoons.com/en/fantasy/tower-of-god/list?title_no=")
            .is_err()
    );
}

#[tokio::test]
async fn webtoon_returns_none_for_nonexistent_id() {
    let client = Client::new();
    let webtoon = client.webtoon(1, Type::Original).await.unwrap();
    assert!(webtoon.is_none());
}

#[tokio::test]
async fn canvas_webtoon_has_no_banner() {
    let client = Client::new();
    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();
    assert!(webtoon.banner().await.unwrap().is_none());
}

#[tokio::test]
async fn original_webtoon_has_no_thumbnail() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    assert!(webtoon.thumbnail().await.unwrap().is_none());
}

#[tokio::test]
async fn canvas_webtoon_has_no_schedule() {
    let client = Client::new();
    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();
    assert!(webtoon.schedule().await.unwrap().is_none());
}

#[tokio::test]
async fn completed_webtoon_is_completed() {
    let client = Client::new();
    let webtoon = client.webtoon(93, Type::Original).await.unwrap().unwrap();
    assert!(webtoon.is_completed().await.unwrap());
}

#[tokio::test]
async fn ongoing_webtoon_is_not_completed() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    assert!(!webtoon.is_completed().await.unwrap());
}

#[tokio::test]
async fn episode_returns_none_for_nonexistent_number() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(u16::MAX).await.unwrap();
    assert!(episode.is_none());
}

#[tokio::test]
async fn episode_number_matches_requested() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    assert_eq!(1, episode.number());
}

#[tokio::test]
async fn hidden_episode_title_returns_not_viewable() {
    use webtoon::platform::webtoons::error::EpisodeError;
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    // Known hidden episode.
    let episode = webtoon.episode(221).await.unwrap().unwrap();
    assert_matches!(episode.title().await, Err(EpisodeError::NotViewable));
}

#[tokio::test]
async fn episode_with_season_in_title_parses_correctly() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(652).await.unwrap().unwrap();
    assert_eq!(Some(3), episode.season().await.unwrap());
}

#[tokio::test]
async fn episode_without_season_returns_none() {
    let client = Client::new();
    let webtoon = client.webtoon(5515, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    assert_eq!(None, episode.season().await.unwrap());
}

#[tokio::test]
async fn first_episode_matches_episode_one() {
    let client = Client::new();
    let webtoon = client.webtoon(4176, Type::Original).await.unwrap().unwrap();
    let first = webtoon.first_episode().await.unwrap();
    assert_eq!(1, first.number());
    assert!(first.published().is_some());
}

#[tokio::test]
async fn alternate_reader_episode_has_no_length() {
    let client = Client::new();
    let webtoon = client.webtoon(4784, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    assert_eq!(None, episode.length().await.unwrap());
}

#[tokio::test]
async fn episode_views_are_none_without_session() {
    let client = Client::new();
    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();
    let mut episodes = webtoon.episodes().await.unwrap();
    episodes.sort_unstable_by_key(|e| e.number());
    if let Some(episode) = episodes.first() {
        assert!(episode.views().is_none());
    }
}

#[tokio::test]
async fn episode_published_is_none_from_single_episode_fetch() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let episode = webtoon.episode(1).await.unwrap().unwrap();
    assert!(episode.published().is_none());
}

#[tokio::test]
async fn episode_published_is_some_from_episodes_list() {
    let client = Client::new();
    let webtoon = client.webtoon(87, Type::Original).await.unwrap().unwrap();
    let mut episodes = webtoon.episodes().await.unwrap();
    episodes.sort_unstable_by_key(|e| e.number());
    if let Some(episode) = episodes.first() {
        assert!(episode.published().is_some());
        assert!(episode.published().unwrap().year() >= 2014);
    }
}

#[tokio::test]
async fn canvas_single_page_returns_results() {
    let client = Client::new();
    let webtoons = client.canvas(1..=1, Sort::Popularity).await.unwrap();
    assert!(!webtoons.is_empty());
    assert!(webtoons.len() == 20);
}

#[tokio::test]
async fn has_session_false_without_session() {
    let client = Client::new();
    assert!(!client.has_session());
}

#[tokio::test]
async fn has_session_true_with_session() {
    let client = Client::with_session("any-value");
    assert!(client.has_session());
}

#[tokio::test]
async fn invalid_session_is_not_valid() {
    let client = Client::with_session("not-a-real-session");
    assert!(!client.has_valid_session().await.unwrap());
}

#[tokio::test]
async fn creator_returns_none_for_nonexistent_profile() {
    let client = Client::new();
    let creator = client
        .creator("this-profile-does-not-exist-xyz123")
        .await
        .unwrap();
    assert!(creator.is_none());
}

#[tokio::test]
async fn korean_creator_has_no_profile() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let creators = webtoon.creators().await.unwrap();
    if let [creator] = creators.as_slice() {
        assert!(creator.profile().is_none());
        assert!(creator.id().await.unwrap().is_none());
        assert!(creator.followers().await.unwrap().is_none());
        assert!(creator.has_patreon().await.unwrap().is_none());
        assert!(creator.webtoons().await.unwrap().is_none());
    } else {
        unreachable!("Tower of God should have one creator");
    }
}

#[tokio::test]
async fn rss_episodes_have_published_dates() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let rss = webtoon.rss().await.unwrap();
    for episode in rss.episodes() {
        assert!(episode.published().is_some());
        assert!(episode.published().unwrap().year() >= 2014);
    }
}

#[tokio::test]
async fn rss_episodes_are_not_empty() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let rss = webtoon.rss().await.unwrap();
    assert!(!rss.episodes().is_empty());
}

#[tokio::test]
async fn rss_thumbnail_starts_with_expected_host() {
    let client = Client::new();
    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();
    let rss = webtoon.rss().await.unwrap();
    assert!(
        rss.thumbnail()
            .starts_with("https://swebtoon-phinf.pstatic.net"),
    );
}
