use tokio::io::AsyncWriteExt;
use webtoon::platform::webtoons::{
    Client, Type,
    error::{BlockUserError, Error},
    webtoon::post::Posts,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = match std::env::var("WEBTOON_SESSION") {
        Ok(session) if !session.is_empty() => Client::with_session(&session),
        _ => Client::new(),
    };

    let Some(webtoon) = client.webtoon(805407, Type::Canvas).await? else {
        panic!("No webtoon of given id and type exits");
    };

    let episode = webtoon
        .episode(2)
        .await?
        .expect("No episode for given number");

    if client.has_valid_session().await.is_ok_and(|result| result) {
        // Post content and if its marked as a spoiler.
        episode.post("MESSAGE", false).await?;
    }

    println!("Getting posts, could take a while...");
    let posts = episode.posts().await?;
    println!("Posts gotten!");

    println!();

    for post in posts {
        println!("username: {}", post.poster().username());
        println!("contents: {}", post.body().contents());
        println!("is spoiler: {}", post.body().is_spoiler());
        println!("flare: {:?}", post.body().flare());
        println!("upvotes: {}", post.upvotes());
        println!("downvotes: {}", post.downvotes());
        let replies: u32 = post.replies().await?;
        println!("replies: {replies}");
        println!("super like: {:?}", post.poster().super_like());

        for reply in post.replies::<Posts>().await? {
            println!("\tusername: {}", reply.poster().username());
            println!("\tcontents: {}", reply.body().contents());
            println!("\tis spoiler: {}", reply.body().is_spoiler());
            println!("\tflare: {:?}", reply.body().flare());
            println!("\tupvotes: {}", reply.upvotes());
            println!("\tdownvotes: {}", reply.downvotes());
            println!();
        }

        if client.has_session() {
            post.upvote().await?;
            post.downvote().await?;
            post.unvote().await?;

            // If has valid session and user has moderating permission(is the creator)
            match post.poster().block().await {
                Ok(()) => {}
                Err(BlockUserError::InvalidPermissions) => eprintln!("No moderating permissions!"),
                Err(BlockUserError::BlockSelf) => eprintln!("Cannot block self!"),
                Err(err) => eprintln!("{err}"),
            }

            post.reply("REPLY", true).await?;

            // post.delete().await?;
        }

        println!();
    }

    // If memory constraints are an issue, then the previous `posts` can become problematic as it retrieves and stores
    // all posts in memory before returning them to operate on. For this usecase, `posts_for_each` is provided but there
    // are limitations:
    // - Cannot guarantee to be duplicate free
    // - Publish order is not guaranteed

    println!("`for_each`:");
    episode
        .posts_for_each(async |post| {
            // Simulating async io.
            let mut stdout = tokio::io::stdout();
            let post = format!("{post:#?}");
            if let Err(err) = stdout.write_all(post.as_bytes()).await {
                eprintln!("Error writing to stdout: {err}");
            }
        })
        .await?;

    return Ok(());
}
