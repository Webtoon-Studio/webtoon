// NOTE: can get average reviews from the story here at one network request. Otherwise need to go episode by episode.
// https://comic.naver.com/api/curation/list?type=ARTIST&id=155779&page=1&pageSize=15&order=UPDATE
// {
//     "pageInfo": {
//         "totalRows": 1,
//         "pageSize": 15,
//         "indexSize": 10,
//         "page": 1,
//         "endRowNum": 1,
//         "rawPage": 1,
//         "lastPage": 1,
//         "totalPages": 1,
//         "startRowNum": 1,
//         "firstPage": 1,
//         "prevPage": 0,
//         "nextPage": 0
//     },
//     "curationViewList": [
//         {
//             "titleId": 183559,
//             "titleName": "신의 탑",
//             "webtoonLevelCode": "WEBTOON",
//             "thumbnailUrl": "https://image-comic.pstatic.net/webtoon/183559/thumbnail/thumbnail_IMAG21_5f3fec31-5c95-4afe-a73f-3046288edb47.jpg",
//             "displayAuthor": "SIU",
//             "writers": [
//                 {
//                     "id": 155779,
//                     "name": "SIU",
//                     "blogUrl": "https://blog.naver.com/inutero3334"
//                 }
//             ],
//             "painters": [
//                 {
//                     "id": 155779,
//                     "name": "SIU",
//                     "blogUrl": "https://blog.naver.com/inutero3334"
//                 }
//             ],
//             "novelOriginAuthors": [],
//             "synopsis": "자신의 모든 것이었던 소녀를 쫓아 탑에 들어온 소년\n그리고 그런 소년을 시험하는 탑",
//             "chargeYn": "N",
//             "averageStarScore": 9.84056,
//             "finished": false,
//             "adult": false,
//             "bm": false,
//             "up": false,
//             "rest": false,
//             "webtoonLevelUp": false,
//             "bestChallengeLevelUp": false,
//             "potenUp": false,
//             "greatestContest": false,
//             "greatestWinning": false,
//             "publishDescription": "월요웹툰",
//             "articleTotalCount": 652,
//             "lastArticleServiceDate": "24.12.01",
//             "tagList": [
//                 {
//                     "id": 183559,
//                     "tagName": "판타지",
//                     "urlPath": "/webtoon?tab=genre&genre=FANTASY",
//                     "curationType": "GENRE_FANTASY"
//                 },
//                 {
//                     "id": 392,
//                     "tagName": "명작",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=392",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 340,
//                     "tagName": "이능력",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=340",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 337,
//                     "tagName": "배틀",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=337",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 333,
//                     "tagName": "모험",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=333",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 332,
//                     "tagName": "전쟁",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=332",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 265,
//                     "tagName": "액션",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=265",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 227,
//                     "tagName": "성장물",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=227",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 186,
//                     "tagName": "서바이벌",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=186",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 181,
//                     "tagName": "세계관",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=181",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 77,
//                     "tagName": "이능력배틀물",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=77",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 66,
//                     "tagName": "소년왕도물",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=66",
//                     "curationType": "CUSTOM_TAG"
//                 },
//                 {
//                     "id": 53,
//                     "tagName": "먼치킨",
//                     "urlPath": "/curation/list?type=CUSTOM_TAG&id=53",
//                     "curationType": "CUSTOM_TAG"
//                 }
//             ],
//             "genreList": [
//                 {
//                     "type": "FANTASY",
//                     "description": "판타지"
//                 }
//             ],
//             "new": false
//         }
//     ]
// }

use std::io::BufWriter;
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = webtoon::platform::naver::Client::new();

    let webtoon = client
        .webtoon(796534)
        .await?
        .expect("webtoon is known to exist");

    println!("title: {}", webtoon.title());
    println!("summary: {}", webtoon.summary());
    println!("type: {:?}", webtoon.r#type());
    println!("creators: {:?}", webtoon.creators());
    println!("genres: {:?}", webtoon.genres());
    println!("favorites: {}", webtoon.favorites());
    println!("thumbnail: {}", webtoon.thumbnail());
    println!("weekdays: {:?}", webtoon.weekdays());
    println!("is_completed: {}", webtoon.is_completed());

    println!();

    // let file = std::fs::File::create("tog.csv").unwrap();

    // let mut csv = BufWriter::new(file);

    // writeln!(
    //     &mut csv,
    //     "episode,season,rating,scorers,likes,comments,replies"
    // )?;

    for episode in webtoon.episodes().await.unwrap() {
        let number = episode.number();
        let season = episode.season().await.unwrap().unwrap_or(1);
        let rating = episode.rating().await.unwrap().unwrap();
        let scorers = episode.scorers().await.unwrap().unwrap_or(0);
        let likes = episode.likes().await.unwrap();
        let (comments, replies) = episode.comments_and_replies().await.unwrap();

        println!("episode: {number}");
        println!("thumbnail: {}", episode.thumbnail().await.unwrap());
        println!("title: {}", episode.title().await.unwrap());
        println!("season: {season}");
        println!("rating: {rating}");
        println!("scorers: {scorers}");
        // println!("note: {:?}", episode.note().await?);
        println!("published: {:?}", episode.published());
        println!("likes: {likes}");
        println!("comments: {comments}");
        println!("replies: {replies}");

        println!();

        for post in episode.posts().await.unwrap() {
            println!("{post:#?}");
        }

        // writeln!(
        //     &mut csv,
        //     "{number},{season},{rating},{scorers},{likes},{comments},{replies}"
        // )?;
    }

    Ok(())
}
