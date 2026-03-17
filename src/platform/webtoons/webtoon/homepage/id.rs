use chrono::NaiveDate;

use crate::{
    platform::webtoons::webtoon::homepage::Unit,
    stdx::error::{AssumeFor, Assumption, assumption},
};

use super::count;

pub(super) fn views(views: &str) -> Result<u64, Assumption> {
    let views = match views {
        billion if billion.ends_with('M') => count(billion, Unit::Billion, Some(","), Some("M"))?,
        million if million.ends_with("JT") => count(million, Unit::Million, Some(","), Some("JT"))?,
        thousand if thousand.contains('.') => count(thousand, Unit::Thousand, Some("."), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(views)
}

pub(super) fn subscribers(subscribers: &str) -> Result<u64, Assumption> {
    let subscribers = match subscribers {
        million if million.ends_with("JT") => count(million, Unit::Million, Some(","), Some("JT"))?,
        thousand if thousand.contains('.') => count(thousand, Unit::Thousand, Some("."), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(subscribers)
}

pub(super) fn schedule(schedule: &str) -> &str {
    schedule
        .trim_start_matches("Update")
        .trim_start_matches("Baca Tiap")
        .trim_start()
}

pub(super) fn date(date: &str) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%d %m %Y";

    let date = match date {
        jan if jan.contains("Jan") => jan.replace("Jan", "1"),
        feb if feb.contains("Feb") => feb.replace("Feb", "2"),
        mar if mar.contains("Mar") => mar.replace("Mar", "3"),
        apr if apr.contains("Apr") => apr.replace("Apr", "4"),
        may if may.contains("Mei") => may.replace("Mei", "5"),
        jun if jun.contains("Jun") => jun.replace("Jun", "6"),
        jul if jul.contains("Jul") => jul.replace("Jul", "7"),
        aug if aug.contains("Agt") => aug.replace("Agt", "8"),
        sep if sep.contains("Sep") => sep.replace("Sep", "9"),
        oct if oct.contains("Okt") => oct.replace("Okt", "10"),
        nov if nov.contains("Nov") => nov.replace("Nov", "11"),
        dec if dec.contains("Des") => dec.replace("Des", "12"),
        _ => assumption!("indonesian date should only contain 12 months, got: {date}"),
    };

    let date = NaiveDate::parse_from_str(&date, FMT).assumption_for(|err| {
        format!("failed to parse `webtoons.com` Webtoon homepage episode date `{date}` with `{FMT}`, got: {err}")
    })?;
    Ok(date)
}
