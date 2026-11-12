//! Bringing a kept extract up to date from its daily change files (Geofabrik's updates folder beside each extract,
//! https://download.geofabrik.de/technical.html): the files read in order into one set of changes, merged into the
//! extract, the graph built again from the result. A file older than 90 days is replaced whole instead.

mod escape;
mod gzip;
mod merge;
mod osc;
mod state;
mod tags;
mod xml;

pub use gzip::{crc32, gunzip};
pub use merge::{Progress, merge};
pub use osc::Changes;
pub use state::{State, days, path, seconds, state, state_path};
pub use tags::e7;

use crate::{Error, Result};

/// After how many days without an update the extract is downloaded whole again.
pub const FULL_AFTER_DAYS: i64 = 90;

/// Whether data last changed at `data_time` is past the 90 days at `now` (both seconds since 1970).
pub fn needs_full(data_time: i64, now: i64) -> bool {
    now - data_time > FULL_AFTER_DAYS * 86_400
}

/// The changes in change files (gzip-compressed osmChange), in the order given.
pub fn changes(files: &[Vec<u8>]) -> Result<Changes> {
    let mut changes = Changes::new();
    for file in files {
        let xml = gunzip(file)?;
        let text = std::str::from_utf8(&xml).map_err(|_| Error::Format("osmChange text"))?;
        osc::read(text, &mut changes)?;
    }
    Ok(changes)
}
