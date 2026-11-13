//! The second read of a PBF file: only the blocks holding nodes, for the positions of the points the roads use. A
//! position off the earth (past 90 degrees of latitude or 180 of longitude) counts as missing, so it splits its road.

use std::io::{Read, Seek};

use super::collect::RoadWay;
use super::{Progress, Step};
use crate::osm::Point;
use crate::pbf::{Blobs, Block};
use crate::{Error, Result};

/// Every point the roads use, sorted, and the position of each (None where the file lacks it).
pub fn located<R: Read + Seek>(
    blobs: &mut Blobs<R>,
    node_blobs: &[u64],
    ways: &[RoadWay],
    progress: &mut Progress<'_>,
) -> Result<(Vec<i64>, Vec<Option<Point>>)> {
    let mut ids: Vec<i64> = ways
        .iter()
        .flat_map(|way| way.refs.iter().copied())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let mut positions = vec![None; ids.len()];
    for (index, offset) in node_blobs.iter().enumerate() {
        progress(Step::Points, index as u64, node_blobs.len() as u64);
        blobs.seek(*offset)?;
        let blob = blobs
            .next_blob()?
            .ok_or(Error::Format("node block vanished"))?;
        Block::parse(&blob.data)?.positions(|id, at| {
            if let (Ok(found), true) = (ids.binary_search(&id), at.valid()) {
                positions[found] = Some(at);
            }
        })?;
    }
    Ok((ids, positions))
}
