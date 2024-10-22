use webtoon::platform::webtoons::{canvas::Sort, Client, Language};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let webtoons = client.canvas(Language::En, 1..10, Sort::Date).await?;

    for webtoon in webtoons {
        println!("title: {}", webtoon.title().await?);
    }

    Ok(())
}
