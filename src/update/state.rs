//! Replication state files and change file paths as Osmosis lays them out and Geofabrik serves them: a state.txt of
//! Java properties ("sequenceNumber=4321", "timestamp=2026-11-11T20\:21\:42Z"), and each day's change at the
//! sequence number split in three groups of three digits ("000/004/321.osc.gz").

use crate::{Error, Result};

/// A sequence number and the time its data reaches, in seconds since 1970 (UTC).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub sequence: i64,
    pub time: i64,
}

const BAD: Error = Error::Format("replication state");

pub fn state(text: &str) -> Result<State> {
    let (mut sequence, mut time) = (None, None);
    for line in text
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
    {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().replace('\\', "");
        match key.trim() {
            "sequenceNumber" => sequence = Some(value.parse().map_err(|_| BAD)?),
            "timestamp" => time = Some(seconds(&value)?),
            _ => {}
        }
    }
    Ok(State {
        sequence: sequence.ok_or(BAD)?,
        time: time.ok_or(BAD)?,
    })
}

/// The path of a change file below the updates folder, for its sequence number.
pub fn path(sequence: i64) -> String {
    format!(
        "{:03}/{:03}/{:03}.osc.gz",
        sequence / 1_000_000,
        sequence / 1_000 % 1_000,
        sequence % 1_000
    )
}

/// The path of a change file's state.
pub fn state_path(sequence: i64) -> String {
    path(sequence).replace(".osc.gz", ".state.txt")
}

/// Seconds since 1970 of a time written "2026-11-11T20:21:42Z".
pub fn seconds(text: &str) -> Result<i64> {
    let bytes = text.as_bytes();
    let shape = bytes.len() == 20 && [4, 7, 10, 13, 16, 19].map(|at| bytes[at]) == *b"--T::Z";
    let field =
        |from: usize, to: usize| text.get(from..to).and_then(|part| part.parse::<i64>().ok());
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
    if !shape
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return Err(BAD);
    }
    Ok(days(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
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
