mod json;
use chrono::DateTime;
pub use json::*;
use tokio::sync::Mutex;

use crate::platform::webtoons::{errors::EpisodeError, webtoon::episode::Episode, Webtoon};
use std::{collections::HashSet, sync::Arc, time::Duration};

pub async fn scrape(webtoon: &Webtoon) -> Result<Vec<Episode>, EpisodeError> {
    // WARN: There must not be any mutating of episodes while in the HashSet, only inserts.
    #[allow(clippy::mutable_key_type)]
    let mut episodes: HashSet<Episode> = HashSet::new();

    let response = webtoon
        .client
        .get_episodes_dashboard(webtoon, 1)
        .await?
        .text()
        .await?;

    let pages = calculate_max_pages(&response)?;

    let dashboard_episodes = DashboardEpisode::parse(&response)?;

    for episode in dashboard_episodes {
        episodes.insert(Episode {
            webtoon: webtoon.clone(),
            number: episode.metadata.number,
            season: Arc::new(Mutex::new(super::super::episode::season(
                &episode.metadata.title,
            ))),
            title: Arc::new(Mutex::new(Some(episode.metadata.title))),
            published: episode.published.map(|timestamp| {
                DateTime::from_timestamp_millis(timestamp)
                    .expect("webtoons should be using proper timestamps")
            }),
            page: Arc::new(Mutex::new(None)),
            views: Some(episode.metadata.views),
            ad_status: Some(episode.dashboard_status.ad_status()),
            published_status: Some(episode.dashboard_status.into()),
        });
    }

    for page in 2..=pages {
        let response = webtoon
            .client
            .get_episodes_dashboard(webtoon, page)
            .await?
            .text()
            .await?;

        let dashboard_episodes = DashboardEpisode::parse(&response)?;

        for episode in dashboard_episodes {
            episodes.insert(Episode {
                webtoon: webtoon.clone(),
                number: episode.metadata.number,
                season: Arc::new(Mutex::new(super::super::episode::season(
                    &episode.metadata.title,
                ))),
                title: Arc::new(Mutex::new(Some(episode.metadata.title))),
                published: episode.published.map(|timestamp| {
                    DateTime::from_timestamp_millis(timestamp)
                        .expect("webtoons should be using proper timestamps")
                }),
                page: Arc::new(Mutex::new(None)),
                views: Some(episode.metadata.views),
                ad_status: Some(episode.dashboard_status.ad_status()),
                published_status: Some(episode.dashboard_status.into()),
            });
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let mut episodes: Vec<Episode> = episodes.into_iter().collect();

    episodes.sort_unstable_by_key(Episode::number);

    Ok(episodes)
}

fn calculate_max_pages(html: &str) -> Result<u16, EpisodeError> {
    let episodes = DashboardEpisode::parse(html)?;

    if episodes.is_empty() {
        return Ok(0);
    }

    let latest = episodes[0].metadata.number;

    // 10 per page. Gets within -1 of the actual page count if there is overflow
    let min = latest / 10;

    // Checks for overflow chapters that would make an extra page
    // If there is any excess it will at most be one extra page, and so if true, the value becomes `1`
    // later added to the page count from before
    let overflow = u16::from((latest % 10) != 0);

    let pages = min + overflow;

    Ok(pages)
}

// fn url(id: u32, page: u16) -> String {
//     format!("https://www.webtoons.com/*/challenge/dashboardEpisode?titleNo={id}&page={page}")
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     // use crate::http::MockWebtoonClient;
//     use chrono::{TimeZone, Utc};

//     const TEST_STORY_EPISODE_DASHBOARD_HTML: &str =
//         include_str!("./episodes/test/test_story_episode_dashboard.html");

//     // const EVERY_NIGHT_EOPISODE_DASHBOARD_HTML: &str =
//     //     include_str!("./episodes/test/every_night_episode_dashboard.html");

//     const YELLOW_LION_EPISODE_DASHBOARD_JSON: &str =
//         include_str!("./episodes/test/yellow_lion_episode_dashboard.json");

//     // const TENACITY_HEX_ESCAPE: &str =
//     //     include_str!("./episodes/test/tenacity_episode_dashboard_invalid_json.json");

//     #[test]
//     fn should_find_episode_list_line() {
//         let result = find_episode_list_line(TEST_STORY_EPISODE_DASHBOARD_HTML).unwrap();

//         let expected = r#"dashboardEpisodeList: [{ "episode": { "titleNo": 843910, "episodeNo": 1, "episodeSeq": 1, "episodeTitle": "Test Episode", "thumbnailImageUrl": "/20230205_259/1675548143974LDl2s_PNG/cb23891a-dab5-4469-b71d-41696223ac9b2582326130891307613.png", "creatorNote": "test note", "exposureStatus": "PUBLISHED", "adminExposureStatus": "NONE", "episodeCategory": "FREE", "exposed": true, "exposureYmdt": 1675548217000, "likeitCount": 0, "readCount": 4, "reportCount": 0, "registerYmdt": 1675548217000, "modifyYmdt": 1675548368000, "xpiderScore": 0E-16, "removedEpisode": false, "newEpisode": false, "exposedStatus": true, "blindEpisode": false, "webtoonType": "CHALLENGE" }, "exposure": { "titleNo": 843910, "episodeNo": 1, "episodeSeq": 1, "exposureType": "FREE", "exposureYmdt": 1675548217000, "webtoonType": "CHALLENGE", "freeExposedEpisode": true }, "reservationList": [], "dashboardStatus": "PUBLISHED", "commentExposure": "COMMENT_ON", "registerDate": 1675548217000, "exposureDate": 1675548217000, "exposureType": "FREE" }],"#;

//         pretty_assertions::assert_str_eq!(expected, result);
//     }

//     #[test]
//     fn should_parse_line_from_html_page() {
//         let line = r#"dashboardEpisodeList: [{ "episode": { "titleNo": 843910, "episodeNo": 1, "episodeSeq": 1, "episodeTitle": "Test Episode", "thumbnailImageUrl": "/20230205_259/1675548143974LDl2s_PNG/cb23891a-dab5-4469-b71d-41696223ac9b2582326130891307613.png", "creatorNote": "test note", "exposureStatus": "PUBLISHED", "adminExposureStatus": "NONE", "episodeCategory": "FREE", "exposed": true, "exposureYmdt": 1675548217000, "likeitCount": 0, "readCount": 4, "reportCount": 0, "registerYmdt": 1675548217000, "modifyYmdt": 1675548368000, "xpiderScore": 0E-16, "removedEpisode": false, "newEpisode": false, "exposedStatus": true, "blindEpisode": false, "webtoonType": "CHALLENGE" }, "exposure": { "titleNo": 843910, "episodeNo": 1, "episodeSeq": 1, "exposureType": "FREE", "exposureYmdt": 1675548217000, "webtoonType": "CHALLENGE", "freeExposedEpisode": true }, "reservationList": [], "dashboardStatus": "PUBLISHED", "commentExposure": "COMMENT_ON", "registerDate": 1675548217000, "exposureDate": 1675548217000, "exposureType": "FREE" }],"#;
//         let result = parse_line(line).unwrap();

//         let expected = test_service_expected_builder();

//         pretty_assertions::assert_eq!(vec![expected], result);
//     }

//     #[test]
//     fn should_build_url() {
//         let webtoon = Webtoon::series(843910, true);

//         let result = url_builder(webtoon, 1);

//         let expected =
//             "https://www.webtoons.com/en/challenge/dashboardEpisode?titleNo=843910&page=1";

//         pretty_assertions::assert_eq!(expected, result);
//     }

//     #[test]
//     fn should_calculate_max_pages_and_latest_chapter() {
//         let (pages, _) = calculate_max_pages(TEST_STORY_EPISODE_DASHBOARD_HTML).unwrap();

//         pretty_assertions::assert_eq!(1, pages);
//     }

//     fn test_service_expected_builder() -> DashboardEpisodeListRaw {
//         DashboardEpisodeListRaw {
//             comment_exposure: String::from("COMMENT_ON"),
//             dashboard_status: String::from("PUBLISHED"),
//             metadata: Metadata {
//                 creator_note: String::new(),
//                 number: 1,
//                 title: "Test Episode".to_string(),
//                 views: 4,
//                 thumbnail: String::from("/20230205_259/1675548143974LDl2s_PNG/cb23891a-dab5-4469-b71d-41696223ac9b2582326130891307613.png"),
//                 is_published: true,
//             },
//             published: Some(Utc.timestamp_millis_opt(1675548217000).unwrap()),
//             reward_ad_on_date: None,
//             reward_ad_off_date: None,
//         }
//     }

//     // fn every_night_expected_builder() -> DashboardEpisodeListRaw {
//     //     DashboardEpisodeListRaw {
//     //         comment_exposure: String::new(),
//     //         dashboard_status: String::new(),
//     //         metadata: Metadata {
//     //             creator_note: String::new(),
//     //             number: 1,
//     //             title: "I want it to End".to_string(),
//     //             views: 537,

//     //             is_published: true,
//     //         },
//     //         published: Some(Utc.timestamp_millis_opt(1562032801000).unwrap()),
//     //         reward_ad_on_date: None,
//     //         reward_ad_off_date: None,
//     //     }
//     // }

//     #[test]
//     fn should_deserialize_episodes() {
//         let line = r#"dashboardEpisodeList: [{ "episode": { "titleNo": 313328, "episodeNo": 1, "episodeSeq": 1, "episodeTitle": "I want it to End", "thumbnailImageUrl": "/20190702_195/1562031153570CrxDc_JPEG/3d5294ba-9006-4b23-bc4a-f943bd84baa3.jpg", "creatorNote": "Do not kill yourself btw.  It&#39;s okay to be sad but do seek help. I did and you should too.", "exposureStatus": "PUBLISHED", "adminExposureStatus": "NONE", "episodeCategory": "FREE", "exposed": true, "exposureYmdt": 1562032801000, "likeitCount": 26, "readCount": 537, "reportCount": 0, "registerYmdt": 1562032801000, "modifyYmdt": 1626175284000, "exposedStatus": true, "removedEpisode": false, "newEpisode": false, "blindEpisode": false, "webtoonType": "CHALLENGE" }, "exposure": { "titleNo": 313328, "episodeNo": 1, "episodeSeq": 1, "exposureType": "FREE", "exposureYmdt": 1562032801000, "webtoonType": "CHALLENGE", "freeExposedEpisode": true }, "reservationList": [], "dashboardStatus": "PUBLISHED", "commentExposure": "COMMENT_ON", "registerDate": 1562032801000, "exposureDate": 1562032801000, "exposureType": "FREE" }],"#;

//         let cleaned = &line[22..line.len() - 1];

//         let _ = serde_json::from_str::<Vec<DashboardEpisodeListRaw>>(
//             &html_escape::decode_html_entities(cleaned),
//         )
//         .unwrap();
//     }

//     #[test]
//     fn should_deserialize_yellow_lion_json_with_no_invalid_escape_error() {
//         let _ = serde_json::from_str::<Vec<DashboardEpisodeListRaw>>(
//             &YELLOW_LION_EPISODE_DASHBOARD_JSON.replace(r"\'", "'"),
//         )
//         .unwrap();
//     }
// }
