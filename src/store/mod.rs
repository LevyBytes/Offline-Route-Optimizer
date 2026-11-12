//! The graph in a file, so a phone builds it once a day and opens it in a moment: "OROGRAPH", the format's version,
//! where the map came from, each part of the graph in turn, then "END.". A save writes beside the file and renames
//! it into place, so the file is whole or absent; an open checks that every index points inside its list, so a
//! damaged file is refused rather than read.

mod bytes;
mod check;
mod codec;
mod records;

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

use crate::graph::{Graph, Meta};
use crate::{Error, Result};
use bytes::{Put, Take};

const MAGIC: &[u8; 8] = b"OROGRAPH";
const END: &[u8; 4] = b"END.";
/// The file format this library writes and reads.
pub const VERSION: u32 = 1;

/// Write the graph to `path` (through `path` with the extension "part").
pub fn save(graph: &Graph, path: &Path) -> Result<()> {
    let part = path.with_extension("part");
    let mut put = Put::new(BufWriter::with_capacity(1 << 20, File::create(&part)?));
    put.raw(MAGIC)?;
    put.raw(&VERSION.to_le_bytes())?;
    let meta = &graph.meta;
    put.text(&meta.region)?;
    put.raw(&meta.data_time.to_le_bytes())?;
    put.raw(&meta.sequence.to_le_bytes())?;
    put.text(&meta.base_url)?;
    records::put(&mut put, graph)?;
    put.raw(END)?;
    let file = put.out.into_inner().map_err(|error| error.into_error())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&part, path)?;
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
    check::check(&graph)?;
    Ok(graph)
}

/// Where a graph file's map came from and how current it is, without reading the rest.
pub fn meta(path: &Path) -> Result<Meta> {
    read_meta(&mut start(path)?)
}

fn start(path: &Path) -> Result<Take<impl Read>> {
    let size = fs::metadata(path)?.len();
    let mut take = Take::new(BufReader::with_capacity(1 << 20, File::open(path)?), size);
    if &take.raw::<8>()? != MAGIC {
        return Err(Error::Format("not a graph file"));
    }
    match take.u32()? {
        VERSION => Ok(take),
        version => Err(Error::Version(version)),
    }
}

fn read_meta(take: &mut Take<impl Read>) -> Result<Meta> {
    let (region, data_time, sequence, base_url) =
        (take.text()?, take.i64()?, take.i64()?, take.text()?);
    Ok(Meta {
        region,
        data_time,
        sequence,
        base_url,
    })
}
