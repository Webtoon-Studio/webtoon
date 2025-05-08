use webtoon::platform::naver::{
    Client,
    errors::Error,
    webtoon::episode::posts::{Posts, Sort},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client
        .webtoon(838432)
        .await?
        .expect("webtoon is known to exist");

    let episode = webtoon
        .episode(5)
        .await?
        .expect("episode 1 should always exist");

    for post in episode.posts(Sort::Best).await? {
        println!("post: {post:#?}");
        println!();
    }

    episode
        .posts_for_each(Sort::New, async |post| {
            let replies = post.replies::<Posts>();
            if let Ok(replies) = replies.await {
                for reply in replies {
                    println!("reply: {reply:#?}");
                    println!();
                }
            }
        })
        .await?;

    return Ok(());
}
