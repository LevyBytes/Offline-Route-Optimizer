//! The two reads of a PBF file: first its ways and relations (the drivable roads, their names, the turn
//! restrictions, and which blocks hold nodes), then only the node blocks, for the positions of the roads' points.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::points::positions;
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

pub fn collect(path: &Path, progress: &mut Progress<'_>) -> Result<Collected> {
    let size = std::fs::metadata(path)?.len().max(1);
    let mut blobs = Blobs::new(BufReader::with_capacity(1 << 20, File::open(path)?));
    let (mut header, mut ways, mut restrictions, mut node_blobs) =
        (None, Vec::new(), Vec::new(), Vec::new());
    let (mut elements, want) = (
        Vec::new(),
        Kinds {
            nodes: false,
            ways: true,
            relations: true,
        },
    );
    while let Some(blob) = blobs.next_blob()? {
        progress(Step::Roads, blob.offset, size);
        match blob.kind {
            BlobKind::Header => header = Some(decode_header(&blob.data)?),
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
                block.elements(want, &mut elements)?;
                for element in elements.drain(..) {
                    match element {
                        Element::Way(way) => {
                            let Some(road) = road(&way.tags) else {
                                continue;
                            };
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
    let mut ids: Vec<i64> = ways
        .iter()
        .flat_map(|way| way.refs.iter().copied())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let positions = positions(&mut blobs, &node_blobs, &ids, progress)?;
    ways.sort_by_key(|way| way.id);
    Ok(Collected {
        header,
        ways,
        restrictions,
        ids,
        positions,
    })
}
