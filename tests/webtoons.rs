use webtoon::platform::webtoons::{
    Client, Language, Type, canvas::Sort, errors::Error, webtoon::episode::posts::Posts,
};

#[tokio::test]
async fn search() -> Result<(), Error> {
    let client = Client::new();

    let _search = client.search("Universe", Language::En).await.unwrap();

    Ok(())
}

#[tokio::test]
async fn creator() -> anyhow::Result<()> {
    let client = Client::new();

    let creator = client
        .creator("JennyToons", Language::En)
        .await
        .unwrap()
        .unwrap();

    let _username = creator.username();
    let _followers = creator.followers().await.unwrap();
    let has_patreon = creator.has_patreon().await.unwrap();
    let _webtoons = creator.webtoons().await.unwrap();

    assert_eq!(Some(true), has_patreon);

    Ok(())
}

#[tokio::test]
async fn originals_page() -> anyhow::Result<()> {
    let client = Client::new();

    let _webtoons = client.originals(Language::En).await.unwrap();

    Ok(())
}

#[tokio::test]
async fn canvas_page() -> anyhow::Result<()> {
    let client = Client::new();

    let webtoons = client.canvas(Language::En, 1..2, Sort::Date).await.unwrap();

    for _webtoon in webtoons {}

    Ok(())
}

#[tokio::test]
async fn webtoon() -> Result<(), Error> {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client.webtoon_from_url(
        "https://www.webtoons.com/en/canvas/testing-service/list?title_no=843910",
    )?;

    let _title = webtoon.title().await.unwrap();
    let _thumbnail = webtoon.thumbnail().await.unwrap();
    let _banner = webtoon.banner().await.unwrap();
    let _lang = webtoon.language();
    let _creators = webtoon.creators().await.unwrap();
    let _genres = webtoon.genres().await.unwrap();
    let _schedule = webtoon.schedule().await.unwrap();
    let _views = webtoon.views().await.unwrap();
    let _likes = webtoon.likes().await.unwrap();
    let _subscribers = webtoon.subscribers().await.unwrap();
    let _rating = webtoon.rating().await.unwrap();
    let _summary = webtoon.summary().await.unwrap();

    if client.has_session() {
        webtoon.rate(10).await.unwrap();
        webtoon.is_subscribed().await.unwrap();
        webtoon.subscribe().await.unwrap();
        webtoon.unsubscribe().await.unwrap();
    }

    return Ok(());
}

#[tokio::test]
async fn posts() -> Result<(), Error> {
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

    if client.has_session() {
        // Post content and if its marked as a spoiler.
        episode.post("MESSAGE", false).await.unwrap();
    }

    let posts = episode.posts().await.unwrap();

    for post in posts {
        for _reply in post.replies::<Posts>().await.unwrap() {}

        if client.has_session() {
            post.upvote().await.unwrap();
            post.downvote().await.unwrap();
            post.unvote().await.unwrap();

            // TODO: Cannot block self, need to filter only non-self posts.
            // TODO: Add unblock()
            // If has valid session and user has moderating permission(is the creator)
            // post.poster().block().await;

            if !post.is_deleted() {
                post.reply("REPLY", true).await.unwrap();
            }

            let replies = post.replies::<Posts>().await.unwrap();

            for reply in replies {
                // Delete just added reply
                if reply.body().contents() == "REPLY" {
                    reply.delete().await.unwrap();
                }
            }

            // Delete just added post
            if post.body().contents() == "MESSAGE" && !post.is_deleted() {
                post.delete().await.unwrap();
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn download() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client.webtoon(843910, Type::Canvas).await.unwrap().unwrap();

    let episode = webtoon
        .episode(1)
        .await?
        .expect("No episode for given number");

    let panels = episode.download().await?;

    // Save as a single, long image.
    panels.save_single("examples/panels").await.unwrap();
    // Save each individual panel as a separate image.
    panels.save_multiple("examples/panels").await.unwrap();

    Ok(())
}

#[tokio::test]
async fn rss() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client.webtoon(95, Type::Original).await.unwrap().unwrap();

    let _rss = webtoon.rss().await.unwrap();

    Ok(())
}
