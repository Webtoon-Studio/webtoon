use webtoon::platform::webtoons::{Client, errors::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let webtoon = client.webtoon_from_url(
        "https://www.webtoons.com/en/canvas/testing-service/list?title_no=843910",
    )?;

    println!("title: {}", webtoon.title().await?);
    println!("thumbnail: {}", webtoon.thumbnail().await?);
    println!("banner: {:?}", webtoon.banner().await?);
    println!("language {:?}", webtoon.language());
    println!("creators: {:?}", webtoon.creators().await?);
    println!("genres: {:?}", webtoon.genres().await?);
    println!("schedule: {:?}", webtoon.schedule().await?);
    println!("views: {}", webtoon.views().await?);
    println!("likes: {}", webtoon.likes().await?);
    println!("subscribers: {}", webtoon.subscribers().await?);
    println!("rating: {}", webtoon.rating().await?);
    println!("summary: {}", webtoon.summary().await?);

    if client.has_session() {
        webtoon.rate(10).await?;
        webtoon.is_subscribed().await?;
        webtoon.subscribe().await?;
        webtoon.unsubscribe().await?;
    }

    return Ok(());
}
