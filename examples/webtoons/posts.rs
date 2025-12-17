use webtoon::platform::webtoons::{Client, Type, error::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client
        .webtoon(805407, Type::Canvas)
        .await?
        .expect("webtoon is known to exist");

    let episode = webtoon
        .episode(2)
        .await?
        .expect("episode is known to exist");

    let mut comments = episode.posts();

    while let Some(comment) = comments.next().await.unwrap() {
        println!("username: {}", comment.poster().username());
        println!("contents: {}", comment.body().contents());
        println!("is spoiler: {}", comment.body().is_spoiler());
        println!("flare: {:?}", comment.body().flare());
        println!("upvotes: {}", comment.upvotes());
        println!("downvotes: {}", comment.downvotes());
        let replies = comment.reply_count();
        println!("replies: {replies}");
        println!("super like: {:?}", comment.poster().super_like());
        println!("is_top: {}", comment.is_top());

        for reply in comment.replies().await? {
            println!("\tusername: {}", reply.poster().username());
            println!("\tcontents: {}", reply.body().contents());
            println!("\tis spoiler: {}", reply.body().is_spoiler());
            println!("\tflare: {:?}", reply.body().flare());
            println!("\tupvotes: {}", reply.upvotes());
            println!("\tdownvotes: {}", reply.downvotes());
            println!();
        }

        println!();
    }

    return Ok(());
}
