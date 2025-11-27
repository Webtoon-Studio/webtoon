use webtoon::platform::webtoons::{
    Client, Language, Type,
    canvas::Sort,
    meta::Genre,
    originals::{Schedule, Weekday},
    webtoon::post::Posts,
};
// TODO: add coverage for original and canvas genres to make sure there is full coverage.

#[tokio::test]
async fn user_info_should_deserialize_even_with_invalid_session() {
    let client = Client::new();

    let user_info = client
        .user_info_for_session("not-a-real-session")
        .await
        .unwrap();

    assert_eq!(None, user_info.username());
    assert_eq!(None, user_info.profile());
    assert!(!user_info.is_logged_in());
}

#[tokio::test]
async fn english_search() {
    let client = Client::new();
    let _search = client.search("Universe", Language::En).await.unwrap();
}

#[tokio::test]
async fn english_creator() {
    let client = Client::new();

    let creator = client
        .creator("JennyToons", Language::En)
        .await
        .unwrap()
        .unwrap();

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
    let _webtoons = client.originals(Language::En).await.unwrap();
}

#[tokio::test]
async fn english_canvas_page() {
    let client = Client::new();
    let webtoons = client.canvas(Language::En, 1..2, Sort::Date).await.unwrap();
    for _webtoon in webtoons {}
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

    let language = webtoon.language();
    assert_eq!(Language::En, language);

    let schedule = webtoon.schedule().await.unwrap();
    assert!(schedule.is_none());

    let _views = webtoon.views().await.unwrap();

    let _likes = webtoon.likes().await.unwrap();

    let _subscribers = webtoon.subscribers().await.unwrap();

    let summary = webtoon.summary().await.unwrap();
    assert_eq!("test", summary);

    if client.has_valid_session().await.is_ok_and(|result| result) {
        assert!(!webtoon.is_subscribed().await.unwrap());
        webtoon.subscribe().await.unwrap();
        assert!(webtoon.is_subscribed().await.unwrap());
        webtoon.unsubscribe().await.unwrap();
        assert!(!webtoon.is_subscribed().await.unwrap());
    }
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
            "https://swebtoon-phinf.pstatic.net/20230613_266/168661779606648vbe_PNG/5ImTheVillain_landingpage_desktop_fg.png"
        ),
        banner.as_deref()
    );

    let language = webtoon.language();
    assert_eq!(Language::En, language);

    let creators = webtoon.creators().await.unwrap();
    match creators.as_slice() {
        [] => unreachable!("every webtoon must have a creator"),
        [_first, _second, _rest @ ..] => {
            unreachable!("`I Am the Villain` should only have one creator")
        }
        [creator] => {
            assert_eq!("Sejji", creator.username());
            assert_eq!(Some("08x59"), creator.profile());
            assert!(creator.has_patreon().await.unwrap().unwrap());
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

    let likes = webtoon.likes().await.unwrap();
    assert!(likes > 5_333_000, "likes were {likes}");

    let subscribers = webtoon.subscribers().await.unwrap();
    assert!(subscribers >= 1_200_000);

    let summary = webtoon.summary().await.unwrap();
    assert_eq!(
        "Working hard is supposed to take you far - but into the world of your best friend's novel? For Lucy, being whisked into a life of ballrooms and picnics isn't exactly what it's cracked up to be; at least, not when everyone has mistaken her for the villain flagged for death. To escape this fate, Lucy must transform from modern workaholic to high society schemer if she even has a chance at returning home. Will she make it? Or will this world of beautiful outfits, strawberry desserts, and dashingly handsome gentlemen seal her fate?",
        summary
    );

    if client.has_valid_session().await.is_ok_and(|result| result) {
        assert!(!webtoon.is_subscribed().await.unwrap());
        webtoon.subscribe().await.unwrap();
        assert!(webtoon.is_subscribed().await.unwrap());
        webtoon.unsubscribe().await.unwrap();
        assert!(!webtoon.is_subscribed().await.unwrap());
    }
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
    assert_eq!(Some(27437), episode.length().await.unwrap());
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
async fn englsh_canvas_posts() {
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

    if client.has_valid_session().await.is_ok_and(|result| result) {
        // Post content and if its marked as a spoiler.
        episode.post("MESSAGE", false).await.unwrap();
    }

    let posts = episode.posts().await.unwrap();

    for post in posts {
        if client.has_valid_session().await.unwrap()
            && post.poster().is_current_session_user()
            && post.body().contents() == "MESSAGE"
        {
            post.reply("REPLY", true).await.unwrap();

            // Delete just added post
            post.delete().await.unwrap();
        } else {
            for reply in post.replies::<Posts>().await.unwrap() {
                _ = std::hint::black_box(reply);
            }
        }

        if client.has_valid_session().await.unwrap() && post.poster().username() == "Nen19" {
            let (upvotes, downvotes) = post.unvote().await.unwrap();
            assert_eq!(0, upvotes);
            assert_eq!(1, downvotes);

            let (upvotes, downvotes) = post.upvote().await.unwrap();
            assert_eq!(1, upvotes, "{upvotes}, {downvotes}");
            assert_eq!(1, downvotes);

            let (upvotes, downvotes) = post.downvote().await.unwrap();
            assert_eq!(0, upvotes);
            assert_eq!(2, downvotes);

            let (upvotes, downvotes) = post.unvote().await.unwrap();
            assert_eq!(0, upvotes);
            assert_eq!(1, downvotes);
        }
    }

    // Clean up previous reply to post.
    //
    // This is needed as currently cannot get
    // updated replies after getting a `Post`.
    //
    // This refreshes to get new data for posts
    // and therefore can find all replies with `REPLY`.
    for post in episode.posts().await.unwrap() {
        for reply in post.replies::<Posts>().await.unwrap() {
            if reply.body().contents() == "REPLY" {
                reply.delete().await.unwrap();
            }
        }
    }
}

#[tokio::test]
async fn englsh_original_posts() {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client.webtoon(1262, Type::Original).await.unwrap().unwrap();

    let episode = webtoon
        .episode(2)
        .await
        .unwrap()
        .expect("No episode for given number");

    if client.has_valid_session().await.is_ok_and(|result| result) {
        // Post content and if its marked as a spoiler.
        episode.post("MESSAGE", false).await.unwrap();
    }

    let posts = episode.posts().await.unwrap();

    for post in posts {
        for _reply in post.replies::<Posts>().await.unwrap() {}

        if client.has_valid_session().await.is_ok_and(|result| result) {
            post.upvote().await.unwrap();
            post.downvote().await.unwrap();
            post.unvote().await.unwrap();

            // There are some complications around replying and then deleting
            // replies, and as this is currently not a priority, just do simple
            // tests.

            // Delete just added post
            if post.body().contents() == "MESSAGE" && !post.is_deleted() {
                post.delete().await.unwrap();
            }
        }
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
    panels.save_single("tests/panels").await.unwrap();
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
    panels.save_single("tests/panels").await.unwrap();
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
    panels.save_multiple("tests/panels").await.unwrap();
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
    panels.save_multiple("tests/panels").await.unwrap();
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
