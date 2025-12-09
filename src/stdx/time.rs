use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

#[derive(Debug, Clone, Copy)]
pub enum DateOrDateTime {
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
}

impl DateOrDateTime {
    #[inline]
    pub fn day(&self) -> u32 {
        match self {
            Self::Date(naive_date) => naive_date.day(),
            Self::DateTime(date_time) => date_time.day(),
        }
    }

    #[inline]
    pub fn month(&self) -> u32 {
        match self {
            Self::Date(naive_date) => naive_date.month(),
            Self::DateTime(date_time) => date_time.month(),
        }
    }

    #[inline]
    pub fn year(&self) -> i32 {
        match self {
            Self::Date(naive_date) => naive_date.year(),
            Self::DateTime(date_time) => date_time.year(),
        }
    }

    #[inline]
    pub fn hour(&self) -> Option<u32> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(date_time) => Some(date_time.hour()),
        }
    }

    #[inline]
    pub fn minute(&self) -> Option<u32> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(date_time) => Some(date_time.minute()),
        }
    }

    #[inline]
    pub fn second(&self) -> Option<u32> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(date_time) => Some(date_time.second()),
        }
    }

    #[inline]
    pub fn timestamp(&self) -> Option<i64> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(date_time) => Some(date_time.timestamp()),
        }
    }
}

impl From<NaiveDate> for DateOrDateTime {
    #[inline]
    fn from(date: NaiveDate) -> Self {
        Self::Date(date)
    }
}

impl From<DateTime<Utc>> for DateOrDateTime {
    #[inline]
    fn from(datetime: DateTime<Utc>) -> Self {
        Self::DateTime(datetime)
    }
}
