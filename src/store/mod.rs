//! The graph in a file, so a phone builds it once a day and opens it in a moment: "OROGRAPH", the format's version,
//! where the map came from, each part of the graph in turn, then "END.". A save writes beside the file under a name of
//! its own and renames it into place, so the file is whole or absent; an open checks every length against what is
//! left of the file before taking room for it, and that every index points inside its list, so a damaged file is
//! refused rather than read.

mod bytes;
mod check;
mod codec;
mod put;
mod records;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

pub use check::check;

use crate::graph::{Graph, Meta};
use crate::temp::Part;
use crate::{Error, Result};
use bytes::Take;
use put::Put;

const MAGIC: &[u8; 8] = b"OROGRAPH";
const END: &[u8; 4] = b"END.";
/// The longest region name and updates address the file keeps.
const MOST_REGION: usize = 1 << 10;
const MOST_URL: usize = 4 << 10;
/// The file format this library writes and reads.
pub const VERSION: u32 = 1;

/// Write the graph to `path` (through a part file beside it, removed if the save fails).
pub fn save(graph: &Graph, path: &Path) -> Result<()> {
    let (part, file) = Part::new(path)?;
    let mut put = Put::new(BufWriter::with_capacity(1 << 20, file));
    put.raw(MAGIC)?;
    put.raw(&VERSION.to_le_bytes())?;
    let meta = &graph.meta;
    put.text(&meta.region, MOST_REGION)?;
    put.raw(&meta.data_time.to_le_bytes())?;
    put.raw(&meta.sequence.to_le_bytes())?;
    put.text(&meta.base_url, MOST_URL)?;
    records::put(&mut put, graph)?;
    put.raw(END)?;
    let file = put.out.into_inner().map_err(|error| error.into_error())?;
    file.sync_all()?;
    drop(file);
    part.finish(path)?;
    Ok(())
}

/// The graph a file holds, checked.
pub fn open(path: &Path) -> Result<Graph> {
    let mut take = start(path)?;
    let meta = read_meta(&mut take)?;
    let mut graph = records::take(&mut take)?;
    graph.meta = meta;
    if &take.raw::<4>()? != END {
        return Err(Error::Format("graph file end"));
    }
    check(&graph)?;
    Ok(graph)
}

/// Where a graph file's map came from and how current it is, without reading the rest.
pub fn meta(path: &Path) -> Result<Meta> {
    read_meta(&mut start(path)?)
}

fn start(path: &Path) -> Result<Take<impl Read>> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    let mut take = Take::new(BufReader::with_capacity(1 << 20, file), size);
    if &take.raw::<8>()? != MAGIC {
        return Err(Error::Format("not a graph file"));
    }
    match take.u32()? {
        VERSION => Ok(take),
        version => Err(Error::Version(version)),
    }
}

fn read_meta(take: &mut Take<impl Read>) -> Result<Meta> {
    let (region, data_time, sequence, base_url) = (
        take.text(MOST_REGION)?,
        take.i64()?,
        take.i64()?,
        take.text(MOST_URL)?,
    );
    Ok(Meta {
        region,
        data_time,
        sequence,
        base_url,
    })
}
