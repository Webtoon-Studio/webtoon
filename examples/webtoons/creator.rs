use anyhow::bail;
use webtoon::platform::webtoons::{Client, Language};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let Some(creator) = client.creator("w7m5o", Language::En).await? else {
        bail!("no creator exists with given id");
    };

    println!("{}", creator.username());
    println!("{:?}", creator.followers().await?);
    println!("{:#?}", creator.webtoons().await?);

    Ok(())
}
