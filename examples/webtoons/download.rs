use webtoon::platform::webtoons::{Client, Type, error::Error};

// NOTE: To run: `cargo run --example download --features download`
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
        panic!("No webtoon of given id and type exits");
    };

    let episode = webtoon
        .episode(1)
        .await?
        .expect("No episode for given number");

    let panels = episode.download().await?;

    // Save as a single, long image.
    panels.save_single("examples/panels").await?;
    // Save each individual panel as a separate image.
    panels.save_multiple("examples/panels").await?;

    return Ok(());
}
