//! Bringing a kept extract up to date from its daily change files (Geofabrik's updates folder beside each extract,
//! <https://download.geofabrik.de/technical.html>): the files read in order into one set of changes, merged into the
//! extract, the graph built again from the result. A file older than 90 days is replaced whole instead.

mod escape;
mod gzip;
mod merge;
mod osc;
mod state;
mod tags;
mod time;
mod xml;

pub use gzip::{LIMIT, crc32, gunzip, gunzip_within};
pub use merge::{Progress, merge};
pub use osc::Changes;
pub use state::{State, path, state, state_path};
pub use tags::e7;
pub use time::{days, seconds};

use crate::{Error, Result};

/// After how many days without an update the extract is downloaded whole again.
pub const FULL_AFTER_DAYS: i64 = 90;

/// The most text `changes` inflates from all its files together (well over a year of Nevada's days); past it a
/// whole new copy of the extract is the better way.
pub const MOST_CHANGES: usize = 512 << 20;

/// Whether data last changed at `data_time` is past the 90 days at `now` (both seconds since 1970). A data time after
/// `now`, as on a phone whose clock is behind, counts as recent.
pub fn needs_full(data_time: i64, now: i64) -> bool {
    now.saturating_sub(data_time) > FULL_AFTER_DAYS * 86_400
}

/// The changes in change files (gzip-compressed osmChange), in the order given; refused (`Error::Limit`) past
/// `MOST_CHANGES` of text together, or `LIMIT` in one file.
pub fn changes(files: &[Vec<u8>]) -> Result<Changes> {
    changes_within(files, MOST_CHANGES)
}

/// As `changes`, with another limit on the text of all the files together.
pub fn changes_within(files: &[Vec<u8>], most: usize) -> Result<Changes> {
    let (mut changes, mut left) = (Changes::new(), most);
    for file in files {
        let xml = gunzip_within(file, left.min(LIMIT))?;
        left -= xml.len();
        let text = std::str::from_utf8(&xml).map_err(|_| Error::Format("osmChange text"))?;
        osc::read(text, &mut changes)?;
    }
    Ok(changes)
}
