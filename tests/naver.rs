use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::posts::Posts};

#[tokio::test]
async fn creator() -> anyhow::Result<()> {
    let client = Client::new();

    let creator = client.creator("_n41b8i").await.unwrap().unwrap();
    let username = creator.username();
    assert_eq!("호리", username);
    let _profile = creator.profile();
    let id = creator.id().await?;
    assert_eq!(Some("n41b8i"), id.as_deref());
    let _followers = creator.followers().await.unwrap();
    let _webtoons = creator.webtoons().await.unwrap();

    Ok(())
}

#[tokio::test]
async fn webtoon() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client
        .webtoon_from_url("https://comic.naver.com/webtoon/list?titleId=838432")
        .await?
        .expect("no webtoon with the given url exists");

    println!("title: {}", webtoon.title());
    println!("thumbnail: {}", webtoon.thumbnail());
    println!("genres: {:?}", webtoon.genres());
    println!("schedule: {:?}", webtoon.schedule());
    println!("is_completed: {}", webtoon.is_completed());
    println!("is_new: {}", webtoon.is_new());
    println!("is_on_hiatus: {}", webtoon.is_on_hiatus());
    println!("is_featured: {}", webtoon.is_featured());
    println!("is_best_challenge: {}", webtoon.is_best_challenge());
    println!("is_challenge: {}", webtoon.is_challenge());
    println!("likes: {}", webtoon.likes().await?);
    println!("favorites: {}", webtoon.favorites());
    println!("rating: {}", webtoon.rating().await?);
    println!("summary: {}", webtoon.summary());
    println!("creators: {:?}", webtoon.creators());

    Ok(())
}

#[tokio::test]
async fn webtoon_shouldnt_exist() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client.webtoon(1).await?;
    if webtoon.is_some() {
        unreachable!("no webtoon with id `1` should exists");
    }

    Ok(())
}

#[tokio::test]
async fn episode() -> Result<(), Error> {
    let client = Client::new();

    let Some(webtoon) = client.webtoon(838432).await? else {
        panic!("No webtoon of given id exits");
    };

    let episode = webtoon
        .episode(1)
        .await?
        .expect("Episode 1 should always exist");

    println!("title: {}", episode.title().await?);
    println!("number: {}", episode.number());
    println!("season: {:?}", episode.season().await?);
    println!("likes: {}", episode.likes().await?);
    let (comments, replies) = episode.comments_and_replies().await?;
    println!("comments: {comments}");
    println!("replies: {replies}");
    println!("length: {:?}", episode.length().await?);

    return Ok(());
}

#[tokio::test]
async fn posts() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client
        .webtoon(838432)
        .await?
        .expect("webtoon is known to exist");

    let episode = webtoon
        .episode(1)
        .await?
        .expect("episode 1 should always exist");

    for post in episode.posts().await? {
        println!("post: {post:#?}");
        println!();
    }

    episode
        .posts_for_each(async |post| {
            let replies = post.replies::<Posts>();
            if let Ok(replies) = replies.await {
                for reply in replies {
                    println!("reply: {reply:#?}");
                    println!();
                }
            }
        })
        .await?;

    return Ok(());
}

#[ignore = "#102"]
#[tokio::test]
async fn download_single() -> Result<(), Error> {
    let client = Client::new();

    let Some(webtoon) = client.webtoon(838432).await? else {
        panic!("No webtoon of given id exits");
    };

    let episode = webtoon
        .episode(1)
        .await?
        .expect("Episode 1 should always exist");

    let panels = episode.download().await?;

    // Save as a single, long image.
    panels.save_single("tests/panels").await?;

    return Ok(());
}

#[ignore = "#102"]
#[tokio::test]
async fn download_multi() -> Result<(), Error> {
    let client = Client::new();

    let Some(webtoon) = client.webtoon(838432).await? else {
        panic!("No webtoon of given id exits");
    };

    let episode = webtoon
        .episode(1)
        .await?
        .expect("Episode 1 should always exist");

    let panels = episode.download().await?;

    // Save each individual panel as a separate image.
    panels.save_multiple("tests/panels").await?;

    return Ok(());
}
