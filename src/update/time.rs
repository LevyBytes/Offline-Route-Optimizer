//! Times as replication state files write them ("2026-11-11T20:21:42Z", always UTC), in seconds since 1970.

use super::state::digits;
use crate::{Error, Result};

const BAD: Error = Error::Format("replication state");

/// Seconds since 1970 of a time written "2026-11-11T20:21:42Z": digits only in each field, and a day its month has.
pub fn seconds(text: &str) -> Result<i64> {
    let bytes = text.as_bytes();
    let shape = bytes.len() == 20 && [4, 7, 10, 13, 16, 19].map(|at| bytes[at]) == *b"--T::Z";
    let field = |from: usize, to: usize| text.get(from..to).and_then(digits);
    let parts = [
        field(0, 4),
        field(5, 7),
        field(8, 10),
        field(11, 13),
        field(14, 16),
        field(17, 19),
    ];
    let [
        Some(year),
        Some(month),
        Some(day),
        Some(hour),
        Some(minute),
        Some(second),
    ] = parts
    else {
        return Err(BAD);
    };
    let month_ok = (1..=12).contains(&month) && (1..=month_days(year, month)).contains(&day);
    if !shape || !month_ok || hour > 23 || minute > 59 || second > 60 {
        return Err(BAD);
    }
    Ok(days(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// The days in a month of the Gregorian calendar.
fn month_days(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a date in the proleptic Gregorian calendar (Howard Hinnant's days_from_civil).
pub fn days(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}
