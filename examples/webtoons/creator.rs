use webtoon::platform::webtoons::{Client, error::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let creator = client.creator("JennyToons").await?.expect("known to exist");

    println!("id: {:?}", creator.id().await?);
    println!("username: {}", creator.username());
    println!("followers: {:?}", creator.followers().await?);
    println!("has_patreon: {:?}", creator.has_patreon().await?);
    println!("webtoons: {:#?}", creator.webtoons().await?);

    Ok(())
}
