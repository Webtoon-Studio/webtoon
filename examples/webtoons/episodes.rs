use webtoon::platform::webtoons::{errors::Error, Client, Type};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let Some(webtoon) = client.webtoon(843910, Type::Canvas).await? else {
        panic!("No webtoon of given id and type exits");
    };

    let episodes = webtoon.episodes().await?;

    for episode in episodes {
        println!("title: {}", episode.title().await?);
        println!("thumbnail: {}", episode.thumbnail().await?);
        println!("published status: {:?}", episode.published_status());
        println!("ad status: {:?}", episode.ad_status());
        println!("season: {:?}", episode.season().await?);
        println!("episode: {}", episode.number());
        println!("published: {:?}", episode.published());
        println!("likes: {}", episode.likes().await?);
        let (comments, replies) = episode.comments_and_replies().await?;
        println!("comments: {comments}\nreplies: {replies}");
        println!("length: {}", episode.length().await?);
        println!("note: {:?}", episode.note().await?);
        println!();

        if let Ok(true) = client.has_valid_session().await {
            episode.like().await?;
            episode.unlike().await?;
        }
    }

    return Ok(());
}
