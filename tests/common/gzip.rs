//! A gzip member the way Geofabrik's change files are written, for the update tests.

use offline_route_optimizer::update;

/// `data` compressed as one gzip member, with its CRC-32 and length.
pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    out.extend(miniz_oxide::deflate::compress_to_vec(data, 6));
    out.extend(update::crc32(data).to_le_bytes());
    out.extend((data.len() as u32).to_le_bytes());
    out
}
