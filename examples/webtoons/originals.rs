use anyhow::Context;
use chrono::Duration;
use webtoon::platform::webtoons::{Client, Language};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let webtoons = client.originals(Language::En).await?;

    let thirty_days_ago = chrono::Utc::now() - Duration::days(30);

    for webtoon in webtoons {
        println!("Checking `{}`", webtoon.title().await?);
        // Need to use `episodes` as only this function can result in `published` yielding `Some`.
        // TODO: Add a specialization for the first episode, `first_episode`, that will optimize getting
        // first episode data so that this check can be done faster.
        let episodes = webtoon.episodes().await?;

        let first_episode = episodes.episode(1).with_context(|| {
            format!("`{}` didnt have an episode 1: {episodes:#?}", webtoon.id())
        })?;

        // Check for all webtoons who's first episode was published within the last 30 days.
        if first_episode.published() >= Some(thirty_days_ago.timestamp_millis()) {
            println!("New Webtoon! `{}`", webtoon.title().await?);
        }
    }

    Ok(())
}
