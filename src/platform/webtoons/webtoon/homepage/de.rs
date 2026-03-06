use chrono::NaiveDate;

use crate::{
    platform::webtoons::webtoon::homepage::Unit,
    stdx::error::{AssumeFor, Assumption},
};

use super::count;

pub(super) fn views(views: &str) -> Result<u64, Assumption> {
    let views = match views {
        billion if billion.ends_with('B') => count(billion, Unit::Billion, Some(','), Some('B'))?,
        million if million.ends_with('M') => count(million, Unit::Million, Some(','), Some('M'))?,
        thousand if thousand.contains('.') => count(thousand, Unit::Thousand, Some('.'), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(views)
}

pub(super) fn subscribers(subscribers: &str) -> Result<u64, Assumption> {
    let subscribers = match subscribers {
        million if million.ends_with('M') => count(million, Unit::Million, Some(','), Some('M'))?,
        thousand if thousand.contains('.') => count(thousand, Unit::Thousand, Some('.'), None)?,
        hundred => count(hundred, Unit::Hundred, None, None)?,
    };
    Ok(subscribers)
}

pub(super) fn date(date: &str) -> Result<NaiveDate, Assumption> {
    const FMT: &str = "%d.%m.%Y";

    let date = NaiveDate::parse_from_str(date, FMT).assumption_for(|err| {
        format!("failed to parse `webtoons.com` Webtoon homepage episode date `{date}` with `{FMT}`, got: {err}")
    })?;
    Ok(date)
}
