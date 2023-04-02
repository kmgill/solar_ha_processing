extern crate astro;
use astro::*;

use chrono::{NaiveDate, NaiveDateTime};

const SEPTASECONDS_PER_SECOND: u64 = 10000000;
const SEPTASECONDS_PER_MICROSECOND: u64 = 10;
//const SEPTASECONDS_PER_PART_MINUTE : u64 = SEPTASECONDS_PER_DAY * 6;
const SEPTASECONDS_PER_MINUTE: u64 = SEPTASECONDS_PER_SECOND * 60;
const SEPTASECONDS_PER_HOUR: u64 = SEPTASECONDS_PER_SECOND * 60 * 60;
const SEPTASECONDS_PER_DAY: u64 = SEPTASECONDS_PER_HOUR * 24;
const DAYS_PER_400_YEARS: u64 = 303 * 365 + 97 * 366;
//const SEPTASECONDS_PER_400_YEARS : u64 = DAYS_PER_400_YEARS * SEPTASECONDS_PER_DAY;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TimeStamp {
    pub timestamp: u64,
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub microsecond: i32,
}

impl TimeStamp {
    fn is_leap_year(year: u64) -> bool {
        (year % 400 == 0 || year % 4 == 0) && year % 100 != 0
    }

    // This code is adapter from ser_viewer/pipp_timestmp.cpp, (c) 2015 by Chris Garry
    pub fn from_u64(ts_u64: u64) -> TimeStamp {
        let ts = ts_u64 % SEPTASECONDS_PER_DAY;

        let hours = ts / SEPTASECONDS_PER_HOUR;
        let minutes = (ts % SEPTASECONDS_PER_HOUR) / SEPTASECONDS_PER_MINUTE;
        let seconds = (ts % SEPTASECONDS_PER_MINUTE) / SEPTASECONDS_PER_SECOND;
        let microseconds = (ts % SEPTASECONDS_PER_SECOND) / SEPTASECONDS_PER_MICROSECOND;
        let mut days_ts = ts_u64 / SEPTASECONDS_PER_DAY;
        let mut year = 0;

        for y in (1..9999).step_by(400) {
            year = y;
            if days_ts >= DAYS_PER_400_YEARS {
                days_ts -= DAYS_PER_400_YEARS;
            } else {
                break;
            }
        }

        for y in year..9999 {
            let year = y;
            let days_this_year = if TimeStamp::is_leap_year(year) {
                366
            } else {
                365
            };

            if days_ts >= days_this_year {
                days_ts -= days_this_year;
            } else {
                break;
            }
        }

        let mut month = 0;
        for m in 1..13 {
            let days_this_month = match m {
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if TimeStamp::is_leap_year(year) {
                        29
                    } else {
                        28
                    }
                }
                _ => 31,
            };

            month = m;
            if days_ts >= days_this_month {
                days_ts -= days_this_month;
            } else {
                break;
            }
        }

        TimeStamp {
            timestamp: ts_u64,
            year: year as i32,
            month,
            day: days_ts as i32 + 1,
            hour: hours as i32,
            minute: minutes as i32,
            second: seconds as i32,
            microsecond: microseconds as i32,
        }
    }

    pub fn to_julian_day(&self) -> f64 {
        let day_of_month = time::DayOfMonth {
            day: self.day as u8,
            hr: self.hour as u8,
            min: self.minute as u8,
            sec: self.second as f64,
            time_zone: 0.0,
        };

        let date = time::Date {
            year: self.year as i16,
            month: self.month as u8,
            decimal_day: time::decimal_day(&day_of_month),
            cal_type: time::CalType::Gregorian,
        };

        time::julian_day(&date)
    }

    pub fn to_unix_timestamp(&self) -> i64 {
        let date_time: NaiveDateTime =
            NaiveDate::from_ymd_opt(self.year, self.month as u32, self.day as u32)
                .unwrap()
                .and_hms_opt(self.hour as u32, self.minute as u32, self.second as u32)
                .unwrap();
        date_time.timestamp()
    }
}
