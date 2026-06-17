use webtoon::platform::webtoons::{Client, canvas::Sort};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let webtoons = client.canvas(1..10, Sort::Date).await?;

    for webtoon in webtoons {
        println!("title: {}", webtoon.title().await?);
    }

    Ok(())
}
