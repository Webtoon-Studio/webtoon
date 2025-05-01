use webtoon::platform::naver::{Client, errors::Error, webtoon::episode::Sort};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client
        .webtoon(838432)
        .await?
        .expect("webtoon is known to exist");

    for episode in &webtoon.episodes(Sort::Asc).await? {
        println!("title: {}", episode.title().await?);
        println!("thumbnail: {}", episode.thumbnail().await?);
        println!("season: {:?}", episode.season().await?);
        println!("episode: {}", episode.number());
        println!("published: {:?}", episode.published().await?);
        println!("likes: {}", episode.likes().await?);
        let (comments, replies) = episode.comments_and_replies().await?;
        println!("comments: {comments}\nreplies: {replies}");
        // println!("length: {:?}", episode.length().await?);
        println!("note: {:?}", episode.note().await?);
        println!();
    }

    return Ok(());
}
