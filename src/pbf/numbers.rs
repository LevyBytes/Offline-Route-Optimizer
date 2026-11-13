//! Protocol Buffers' number encodings beyond the plain varint: zigzag for signed numbers, packed repeated fields, and
//! the differences PBF files store ids, coordinates and references as.

use super::wire::Wire;
use crate::{Error, Result};

/// The most numbers one packed field may hold: far above what OpenStreetMap's writers put in a block (8,000
/// elements), so a hostile block cannot ask for gigabytes.
pub const MOST_PACKED: usize = 1 << 21;

/// A zigzag-encoded signed number (sint32, sint64).
pub fn zigzag(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

/// The numbers of a packed repeated field, counted (each ends on a byte below 0x80) before room is reserved.
pub fn packed(data: &[u8]) -> Result<Vec<u64>> {
    let count = data.iter().filter(|byte| **byte < 0x80).count();
    if count > MOST_PACKED {
        return Err(Error::Limit("a packed list of more than 2,097,152 numbers"));
    }
    let mut wire = Wire::new(data);
    let mut out = Vec::with_capacity(count);
    while wire.at < data.len() {
        out.push(wire.varint()?);
    }
    Ok(out)
}

/// A packed repeated sint64 field stored as differences from the one before (ids, coordinates, references).
pub fn deltas(data: &[u8]) -> Result<Vec<i64>> {
    let mut sum = 0i64;
    let raw = packed(data)?;
    Ok(raw
        .into_iter()
        .map(|value| {
            sum = sum.wrapping_add(zigzag(value));
            sum
        })
        .collect())
}
