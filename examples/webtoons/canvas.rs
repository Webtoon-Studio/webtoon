use webtoon::platform::webtoons::{Client, canvas::Sort, error::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let webtoons = client.canvas(1..10, Sort::Date).await?;

    for webtoon in webtoons {
        println!("title: {}", webtoon.title().await?);
    }

    Ok(())
}
