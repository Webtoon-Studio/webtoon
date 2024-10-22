use webtoon::platform::webtoons::{errors::Error, Client, Type};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let Some(webtoon) = client.webtoon(95, Type::Original).await? else {
        panic!("No webtoon of given id and type exits");
    };

    let rss = webtoon.rss().await?;

    println!("webtoon url: {}", rss.url());
    println!("title: {}", rss.title());
    println!("summary: {}", rss.summary());
    println!("webtoon thumbnail url: {}", rss.thumbnail());
    println!("creators: {:?}", rss.creators());

    // For more examples on working with an `Episode` check `examples/episodes.rs`.
    for episode in rss.episodes() {
        println!("title: {}", episode.title().await?);
    }

    return Ok(());
}
