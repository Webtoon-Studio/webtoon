use chrono::Duration;
use webtoon::platform::webtoons::{Client, Language};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let webtoons = client.originals(Language::En).await?;

    let thirty_days_ago = chrono::Utc::now() - Duration::days(30);

    for webtoon in webtoons {
        // `first_episode` is a specialized way to get this kind of data
        //  with `published` yielding `Some` where `episode(1)` would yield `None`.
        let first_episode = webtoon.first_episode().await?;

        // Check for all Webtoons who's first episode was published within the last 30 days.
        if first_episode.published() >= Some(thirty_days_ago.timestamp_millis()) {
            println!("New Webtoon! `{}`", webtoon.title().await?);
        }
    }

    Ok(())
}
