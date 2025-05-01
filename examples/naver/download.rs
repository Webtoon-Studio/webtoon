use webtoon::platform::naver::{Client, errors::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
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
    panels.save_single("examples/panels").await?;
    // Save each individual panel as a separate image.
    panels.save_multiple("examples/panels").await?;

    return Ok(());
}
