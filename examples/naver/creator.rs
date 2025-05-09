use anyhow::bail;
use webtoon::platform::naver::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    let Some(creator) = client.creator("_n41b8i").await? else {
        bail!("no creator exists with given id");
    };
    println!("id: {:?}", creator.id().await?);
    println!("username: {}", creator.username());
    println!("followers: {:?}", creator.followers().await?);
    println!("webtoons: {:#?}", creator.webtoons().await?);

    Ok(())
}
