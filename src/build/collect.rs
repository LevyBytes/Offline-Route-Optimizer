//! The two reads of a PBF file: its ways and relations (the drivable roads, their names, the turn restrictions and
//! which blocks hold nodes), then the node blocks for the roads' points. The file opens with its one header block, and
//! the roads hold at most 2^28 points in all (MOST_POINTS), so the graph's counts fit their 32-bit indexes.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::points::located;
use super::{Progress, Step};
use crate::osm::{Element, Point, tag};
use crate::pbf::{BlobKind, Blobs, Block, Header, Kinds, decode_header};
use crate::restrict::{Restriction, restriction};
use crate::roads::{Road, road};
use crate::{Error, Result};

pub struct RoadWay {
    pub id: i64,
    pub refs: Vec<i64>,
    pub road: Road,
    pub name: Option<String>,
}

pub struct Collected {
    pub header: Header,
    pub ways: Vec<RoadWay>,
    pub restrictions: Vec<Restriction>,
    /// Every point the roads use, sorted, and its position (None if the file lacks it).
    pub ids: Vec<i64>,
    pub positions: Vec<Option<Point>>,
}

const MOST_POINTS: usize = 1 << 28;

pub fn collect(path: &Path, progress: &mut Progress<'_>) -> Result<Collected> {
    let file = File::open(path)?;
    let size = file.metadata()?.len().max(1);
    let mut blobs = Blobs::new(BufReader::with_capacity(1 << 20, file));
    let (mut header, mut ways, mut restrictions, mut node_blobs) =
        (None, Vec::new(), Vec::new(), Vec::new());
    let (mut elements, mut points) = (Vec::new(), 0);
    while let Some(blob) = blobs.next_blob()? {
        progress(Step::Roads, blob.offset, size);
        match blob.kind {
            BlobKind::Header if header.is_some() => {
                return Err(Error::Format("a second header block"));
            }
            BlobKind::Header => header = Some(decode_header(&blob.data)?),
            BlobKind::Data if header.is_none() => {
                return Err(Error::Format("data before the header block"));
            }
            BlobKind::Data => {
                let block = Block::parse(&blob.data)?;
                let kinds = block.kinds()?;
                if kinds.nodes {
                    node_blobs.push(blob.offset);
                }
                if !kinds.ways && !kinds.relations {
                    continue;
                }
                elements.clear();
                block.elements(Kinds::LINES, &mut elements)?;
                for element in elements.drain(..) {
                    match element {
                        Element::Way(way) => {
                            let Some(road) = road(&way.tags) else {
                                continue;
                            };
                            points += way.refs.len();
                            if points > MOST_POINTS {
                                return Err(Error::Limit("roads of more than 2^28 points"));
                            }
                            let name = tag(&way.tags, "name").map(str::to_owned);
                            ways.push(RoadWay {
                                id: way.id,
                                refs: way.refs,
                                road,
                                name,
                            });
                        }
                        Element::Relation(relation) => restrictions.extend(restriction(&relation)),
                        Element::Node(_) => {}
                    }
                }
            }
            BlobKind::Other => {}
        }
    }
    let header = header.ok_or(Error::Format("file without a header block"))?;
    let (ids, positions) = located(&mut blobs, &node_blobs, &ways, progress)?;
    ways.sort_by_key(|way| way.id);
    Ok(Collected {
        header,
        ways,
        restrictions,
        ids,
        positions,
    })
}
