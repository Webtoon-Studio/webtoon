use chrono::NaiveDate;

use crate::{
    platform::webtoons::webtoon::homepage::Unit,
    stdx::error::{AssumeFor, Assumption, assumption},
};

use super::count;

pub(super) fn views(views: &str) -> Result<u64, Assumption> {
    let views = match views {
        billion if billion.ends_with('B') => count(billion, Unit::Billion, Some('.'), Some('B'))?,
        million if million.ends_with('M') => count(million, Unit::Million, Some('.'), Some('M'))?,
        thousand if thousand.contains(',') => count(thousand, Unit::Thousand, Some(','), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(views)
}

pub(super) fn subscribers(subscribers: &str) -> Result<u64, Assumption> {
    let subscribers = match subscribers {
        million if million.ends_with('M') => count(million, Unit::Million, Some('.'), Some('M'))?,
        thousand if thousand.contains(',') => count(thousand, Unit::Thousand, Some(','), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(subscribers)
}

pub(super) fn schedule(schedule: &str) -> &str {
    schedule.trim_start_matches("TODOS LOS").trim_start()
}

pub(super) fn date(date: &str) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%d %m %Y";

    let date = match date {
        jan if jan.contains("ene.") => jan.replace("ene.", "1"),
        feb if feb.contains("feb.") => feb.replace("feb.", "2"),
        mar if mar.contains("mar.") => mar.replace("mar.", "3"),
        apr if apr.contains("abr.") => apr.replace("abr.", "4"),
        may if may.contains("may.") => may.replace("may.", "5"),
        jun if jun.contains("jun.") => jun.replace("jun.", "6"),
        jul if jul.contains("jul.") => jul.replace("jul.", "7"),
        aug if aug.contains("ago.") => aug.replace("ago.", "8"),
        sep if sep.contains("sept.") => sep.replace("sept.", "9"),
        oct if oct.contains("oct.") => oct.replace("oct.", "10"),
        nov if nov.contains("nov.") => nov.replace("nov.", "11"),
        dec if dec.contains("dic.") => dec.replace("dic.", "12"),
        _ => assumption!("spanish date should only contain 12 months aprebiated with a `.` suffix"),
    };

    let date = NaiveDate::parse_from_str(&date, FMT).assumption_for(|err| {
        format!("failed to parse `webtoons.com` Webtoon homepage episode date `{date}` with `{FMT}`, got: {err}")
    })?;
    Ok(date)
}
