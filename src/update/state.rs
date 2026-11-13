//! Replication state files and change file paths as Osmosis lays them out and Geofabrik serves them: a state.txt of
//! Java properties ("sequenceNumber=4321", "timestamp=2026-11-11T20\:21\:42Z"), and each day's change at the
//! sequence number split in three groups of three digits ("000/004/321.osc.gz").

use super::time::seconds;
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
            "sequenceNumber" => sequence = Some(digits(&value).ok_or(BAD)?),
            "timestamp" => time = Some(seconds(&value)?),
            _ => {}
        }
    }
    Ok(State {
        sequence: sequence.ok_or(BAD)?,
        time: time.ok_or(BAD)?,
    })
}

/// A number of one to nine decimal digits (no sign, no spaces): a sequence number's range, as its path has nine.
pub(super) fn digits(text: &str) -> Option<i64> {
    let plain = (1..=9).contains(&text.len()) && text.bytes().all(|byte| byte.is_ascii_digit());
    plain.then(|| text.parse().ok()).flatten()
}

/// The path of a change file below the updates folder, for its sequence number (0 to 999,999,999, as `state` reads).
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
