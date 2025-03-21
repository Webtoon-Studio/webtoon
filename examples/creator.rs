use anyhow::bail;
use webtoon::platform::webtoons::{Client, Language};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let Some(creator) = client.creator("JennyToons", Language::En).await? else {
        bail!("no creator exists with given id");
    };

    println!("id: {:?}", creator.id().await?);
    println!("username: {}", creator.username());
    println!("followers: {:?}", creator.followers().await?);
    println!("has_patreon: {:?}", creator.has_patreon().await?);
    println!("webtoons: {:#?}", creator.webtoons().await?);

    Ok(())
}
