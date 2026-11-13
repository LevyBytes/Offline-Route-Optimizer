//! Gzip (RFC 1952) as Geofabrik writes its change files: one member, deflate inside, checked by its CRC-32 and size.

use miniz_oxide::inflate::{TINFLStatus, decompress_to_vec_with_limit};

use crate::{Error, Result};

/// The largest change file `gunzip` inflates (a day of Nevada inflates to under a megabyte).
pub const LIMIT: usize = 128 << 20;

pub fn gunzip(data: &[u8]) -> Result<Vec<u8>> {
    gunzip_within(data, LIMIT)
}

/// As `gunzip`, refusing a file that inflates past `most` bytes.
pub fn gunzip_within(data: &[u8], most: usize) -> Result<Vec<u8>> {
    let bad = |what| Error::Format(what);
    if data.len() < 18 || data[..3] != [0x1f, 0x8b, 8] {
        return Err(bad("not a gzip file"));
    }
    let flags = data[3];
    let mut at = 10;
    if flags & 4 != 0 {
        let extra = data.get(at..at + 2).ok_or(bad("gzip extra field"))?;
        at += 2 + usize::from(u16::from_le_bytes([extra[0], extra[1]]));
    }
    for flag in [8, 16] {
        if flags & flag != 0 {
            let end = data
                .get(at..)
                .and_then(|rest| rest.iter().position(|byte| *byte == 0));
            at += end.ok_or(bad("gzip name"))? + 1;
        }
    }
    if flags & 2 != 0 {
        at += 2;
    }
    let body = data.get(at..data.len() - 8).ok_or(bad("gzip body"))?;
    let out = decompress_to_vec_with_limit(body, most).map_err(|error| match error.status {
        TINFLStatus::HasMoreOutput => Error::Limit("change files inflating past the limit"),
        _ => bad("deflate data"),
    })?;
    let trailer = &data[data.len() - 8..];
    let crc = u32::from_le_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);
    let size = u32::from_le_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]);
    if crc32(&out) != crc || out.len() as u32 != size {
        return Err(bad("gzip check"));
    }
    Ok(out)
}

const TABLE: [u32; 256] = table();

const fn table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
}

/// CRC-32 as gzip and zip use it (ISO 3309, polynomial 0xEDB88320 reflected).
pub fn crc32(data: &[u8]) -> u32 {
    !data.iter().fold(!0u32, |crc, byte| {
        TABLE[((crc ^ u32::from(*byte)) & 0xff) as usize] ^ (crc >> 8)
    })
}
