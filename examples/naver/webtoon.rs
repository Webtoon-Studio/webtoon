use webtoon::platform::naver::{Client, errors::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let webtoon = client.webtoon(1).await?;
    if webtoon.is_some() {
        unreachable!("no webtoon with id `1` should exists");
    }

    let webtoon = client
        .webtoon_from_url("https://comic.naver.com/webtoon/list?titleId=838432")
        .await?
        .expect("no webtoon with the given url exists");

    println!("title: {}", webtoon.title());
    println!("thumbnail: {}", webtoon.thumbnail());
    println!("genres: {:?}", webtoon.genres());
    println!("schedule: {:?}", webtoon.schedule());
    println!("is_completed: {}", webtoon.is_completed());
    println!("is_new: {}", webtoon.is_new());
    println!("is_on_hiatus: {}", webtoon.is_on_hiatus());
    println!("is_featured: {}", webtoon.is_featured());
    println!("is_best_challenge: {}", webtoon.is_best_challenge());
    println!("is_challenge: {}", webtoon.is_challenge());
    println!("likes: {}", webtoon.likes().await?);
    println!("favorites: {}", webtoon.favorites());
    println!("rating: {}", webtoon.rating().await?);
    println!("summary: {}", webtoon.summary());
    println!("creators: {:?}", webtoon.creators());

    return Ok(());
}
