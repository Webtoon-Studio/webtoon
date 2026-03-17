use chrono::NaiveDate;

use crate::{
    platform::webtoons::webtoon::homepage::Unit,
    stdx::error::{AssumeFor, Assumption, assumption},
};

use super::count;

pub(super) fn views(views: &str) -> Result<u64, Assumption> {
    let views = match views {
        billion if billion.ends_with("B") => count(billion, Unit::Billion, Some(","), Some("B"))?,
        million if million.ends_with("M") => count(million, Unit::Million, Some(","), Some("M"))?,
        thousand if thousand.contains("&nbsp;") => {
            count(thousand, Unit::Thousand, Some("&nbsp;"), None)?
        }
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(views)
}

pub(super) fn subscribers(subscribers: &str) -> Result<u64, Assumption> {
    let subscribers = match subscribers {
        million if million.ends_with("M") => count(million, Unit::Million, Some(","), Some("M"))?,
        thousand if thousand.contains("&nbsp;") => {
            count(thousand, Unit::Thousand, Some("&nbsp;"), None)?
        }
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(subscribers)
}

pub(super) fn schedule(schedule: &str) -> &str {
    schedule.trim_start_matches("TOUS LES").trim_start()
}

pub(super) fn date(date: &str) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%d %m %Y";

    let date = match date {
        jan if jan.contains("janv.") => jan.replace("janv.", "1"),
        feb if feb.contains("févr.") => feb.replace("févr.", "2"),
        mar if mar.contains("mars") => mar.replace("mars", "3"),
        apr if apr.contains("avr.") => apr.replace("avr.", "4"),
        may if may.contains("mai") => may.replace("mai", "5"),
        jun if jun.contains("juin") => jun.replace("juin", "6"),
        jul if jul.contains("juil.") => jul.replace("juil.", "7"),
        aug if aug.contains("août") => aug.replace("août", "8"),
        sep if sep.contains("sept.") => sep.replace("sept.", "9"),
        oct if oct.contains("oct.") => oct.replace("oct.", "10"),
        nov if nov.contains("nov.") => nov.replace("nov.", "11"),
        dec if dec.contains("déc.") => dec.replace("déc.", "12"),
        _ => assumption!(
            "french date should only contain 12 months aprebiated with a `.` suffix, got: {date}"
        ),
    };

    let date = NaiveDate::parse_from_str(&date, FMT).assumption_for(|err| {
        format!("failed to parse `webtoons.com` Webtoon homepage episode date `{date}` with `{FMT}`, got: {err}")
    })?;
    Ok(date)
}
