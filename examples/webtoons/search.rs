use webtoon::platform::webtoons::{Client, error::Error};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let timer = std::time::Instant::now();

    let query = "Universe";
    println!("Searching: {query}");
    let search = client.search(query).await?;

    println!(
        "Took: {}ms for `{}` results",
        timer.elapsed().as_millis(),
        search.len()
    );

    println!("Found:");
    for webtoon in search {
        println!("  - {}", webtoon.title());
    }

    Ok(())
}
