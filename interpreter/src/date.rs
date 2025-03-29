use chrono::{
    DateTime, Datelike, FixedOffset, LocalResult, Offset, TimeZone, Timelike, Utc, Weekday,
};
use chrono_tz::Tz;
use std::ops::{Add, Sub};

use crate::runtime_error::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    ts: i64,
    tz: FixedOffset,
}

impl Date {
    pub fn from_timestamp_millis(ts: i64, tz: Option<i32>) -> Result<Self, Box<RuntimeError>> {
        let tz = tz.map(|offset| FixedOffset::east_opt(offset).unwrap());

        match Utc.timestamp_millis_opt(ts) {
            LocalResult::Single(_) => Ok(Self {
                ts,
                tz: tz.unwrap_or(FixedOffset::east_opt(0).unwrap()),
            }),
            _ => Err(Box::new(RuntimeError::InvalidTimestamp {
                timestamp: ts,
                span: None,
            })),
        }
    }

    pub fn from_iso_string(s: &str) -> Result<Self, Box<RuntimeError>> {
        let fixed_dt = match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(dt) => dt,
            Err(e) => {
                return Err(Box::new(RuntimeError::DateIsoParseError {
                    source: e,
                    span: None,
                }))
            }
        };

        let utc_ts = fixed_dt.with_timezone(&Utc).timestamp_millis();
        let offset = fixed_dt.offset().fix();

        Ok(Self {
            ts: utc_ts,
            tz: offset,
        })
    }

    pub fn with_named_timezone(&self, tz_str: &str) -> Result<Self, Box<RuntimeError>> {
        let named_zone = match tz_str.parse::<Tz>() {
            Ok(z) => z,
            Err(_) => {
                return Err(Box::new(RuntimeError::InvalidTimeZoneString {
                    tz_str: tz_str.into(),
                    span: None,
                }));
            }
        };

        let utc_dt = Utc.timestamp_millis_opt(self.ts).single().unwrap();
        let dt_in_zone = utc_dt.with_timezone(&named_zone);
        let new_offset = dt_in_zone.offset().fix();

        Ok(Self {
            ts: self.ts,
            tz: new_offset,
        })
    }

    pub fn to_offset_string(&self) -> String {
        let total_seconds = self.tz.local_minus_utc();

        let sign = if total_seconds >= 0 { '+' } else { '-' };
        let secs = total_seconds.abs();

        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;

        format!("{}{:02}:{:02}", sign, hours, minutes)
    }

    pub fn to_iso_string(&self) -> String {
        let dt_tz = self.as_datetime_tz();
        dt_tz.to_rfc3339()
    }

    pub fn to_timestamp_millis(&self) -> i64 {
        self.ts
    }

    pub fn year(&self) -> i32 {
        self.as_datetime_tz().year()
    }

    pub fn month(&self) -> u32 {
        self.as_datetime_tz().month()
    }

    pub fn day(&self) -> u32 {
        self.as_datetime_tz().day()
    }

    pub fn hour(&self) -> u32 {
        self.as_datetime_tz().hour()
    }

    pub fn minute(&self) -> u32 {
        self.as_datetime_tz().minute()
    }

    pub fn second(&self) -> u32 {
        self.as_datetime_tz().second()
    }

    pub fn weekday(&self) -> u8 {
        match self.as_datetime_tz().weekday() {
            Weekday::Sun => 0,
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
        }
    }

    pub fn ordinal(&self) -> u32 {
        self.as_datetime_tz().ordinal()
    }

    pub fn iso_week(&self) -> u32 {
        self.as_datetime_tz().iso_week().week()
    }

    pub fn is_leap_year(&self) -> bool {
        let year = self.year();
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn as_datetime_tz(&self) -> DateTime<FixedOffset> {
        let utc_dt = Utc.timestamp_millis_opt(self.ts).single().unwrap();

        utc_dt.with_timezone(&self.tz)
    }
}

impl Add<i64> for Date {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Date {
            ts: self.ts + rhs,
            tz: self.tz,
        }
    }
}

impl Sub<i64> for Date {
    type Output = Self;

    fn sub(self, rhs: i64) -> Self::Output {
        Date {
            ts: self.ts - rhs,
            tz: self.tz,
        }
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ts.cmp(&other.ts)
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ts.cmp(&other.ts))
    }
}
