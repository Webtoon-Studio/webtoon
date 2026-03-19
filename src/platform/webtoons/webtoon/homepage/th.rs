use chrono::NaiveDate;

use crate::{
    platform::webtoons::webtoon::homepage::Unit,
    stdx::error::{AssumeFor, Assumption, assumption},
};

use super::count;

pub(super) fn views(views: &str) -> Result<u64, Assumption> {
    let views = match views {
        billion if billion.ends_with("B") => count(billion, Unit::Billion, Some("."), Some("B"))?,
        million if million.ends_with("M") => count(million, Unit::Million, Some("."), Some("M"))?,
        thousand if thousand.contains(",") => count(thousand, Unit::Thousand, Some(","), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(views)
}

pub(super) fn subscribers(subscribers: &str) -> Result<u64, Assumption> {
    let subscribers = match subscribers {
        million if million.ends_with("M") => count(million, Unit::Million, Some("."), Some("M"))?,
        thousand if thousand.contains(",") => count(thousand, Unit::Thousand, Some(","), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(subscribers)
}

pub(super) fn schedule(schedule: &str) -> &str {
    schedule.trim_start_matches("ทุกๆ").trim_start()
}

pub(super) fn date(date: &str) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%d %m %Y";

    let date = match date {
        jan if jan.contains("ม.ค.") => jan.replace("ม.ค.", "1"),
        feb if feb.contains("ก.พ.") => feb.replace("ก.พ.", "2"),
        mar if mar.contains("มี.ค.") => mar.replace("มี.ค.", "3"),
        apr if apr.contains("เม.ย.") => apr.replace("เม.ย.", "4"),
        may if may.contains("พ.ค.") => may.replace("พ.ค.", "5"),
        jun if jun.contains("มิ.ย.") => jun.replace("มิ.ย.", "6"),
        jul if jul.contains("ก.ค.") => jul.replace("ก.ค.", "7"),
        aug if aug.contains("ส.ค.") => aug.replace("ส.ค.", "8"),
        sep if sep.contains("ก.ย.") => sep.replace("ก.ย.", "9"),
        oct if oct.contains("ต.ค.") => oct.replace("ต.ค.", "10"),
        nov if nov.contains("พ.ย.") => nov.replace("พ.ย.", "11"),
        dec if dec.contains("ธ.ค.") => dec.replace("ธ.ค.", "12"),
        _ => assumption!(
            "thai date should only contain 12 months aprebiated with a `.` suffix, got: {date}"
        ),
    };

    let date = NaiveDate::parse_from_str(&date, FMT).assumption_for(|err| {
        format!("failed to parse `webtoons.com` Webtoon homepage episode date `{date}` with `{FMT}`, got: {err}")
    })?;
    Ok(date)
}
