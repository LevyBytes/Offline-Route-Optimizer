//! The second read of a PBF file: only the blocks holding nodes, for the positions of the points the roads use.

use std::io::{Read, Seek};

use super::{Progress, Step};
use crate::osm::Point;
use crate::pbf::{Blobs, Block};
use crate::{Error, Result};

/// The position of each point in `ids` (sorted), None where the file lacks it.
pub fn positions<R: Read + Seek>(
    blobs: &mut Blobs<R>,
    node_blobs: &[u64],
    ids: &[i64],
    progress: &mut Progress<'_>,
) -> Result<Vec<Option<Point>>> {
    let mut positions = vec![None; ids.len()];
    for (index, offset) in node_blobs.iter().enumerate() {
        progress(Step::Points, index as u64, node_blobs.len() as u64);
        blobs.seek(*offset)?;
        let blob = blobs
            .next_blob()?
            .ok_or(Error::Format("node block vanished"))?;
        Block::parse(&blob.data)?.positions(|id, at| {
            if let Ok(found) = ids.binary_search(&id) {
                positions[found] = Some(at);
            }
        })?;
    }
    Ok(positions)
}
