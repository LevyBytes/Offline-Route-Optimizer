//! OpenStreetMap's PBF format, read and written by this library itself
//! (https://wiki.openstreetmap.org/wiki/PBF_Format): only zlib-compressed and raw blocks, which is what Geofabrik
//! and the planet files use.

mod blob;
mod block;
mod dense;
mod encode;
mod frame;
mod header;
mod items;
mod kinds;
mod numbers;
mod out;
mod scan;
mod strings;
mod wire;
mod write;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub use blob::{Blob, BlobKind, Blobs};
pub use block::Block;
pub use header::Header;
pub use kinds::Kinds;
pub use write::Writer;

use crate::osm::Element;
use crate::{Error, Result};

/// A file's header block (its first blob).
pub fn read_header(path: &Path) -> Result<Header> {
    let mut blobs = Blobs::new(BufReader::new(File::open(path)?));
    match blobs.next_blob()? {
        Some(blob) if blob.kind == BlobKind::Header => header::decode(&blob.data),
        _ => Err(Error::Format("file does not start with a header block")),
    }
}

/// A header block's contents.
pub fn decode_header(data: &[u8]) -> Result<Header> {
    header::decode(data)
}

/// A file's header and every element in it, in its order (for small files: tests and checks).
pub fn elements(path: &Path) -> Result<(Header, Vec<Element>)> {
    let (mut header, mut elements) = (None, Vec::new());
    let mut blobs = Blobs::new(BufReader::new(File::open(path)?));
    while let Some(blob) = blobs.next_blob()? {
        match blob.kind {
            BlobKind::Header => header = Some(header::decode(&blob.data)?),
            BlobKind::Data => Block::parse(&blob.data)?.elements(Kinds::ALL, &mut elements)?,
            BlobKind::Other => {}
        }
    }
    Ok((
        header.ok_or(Error::Format("file without a header block"))?,
        elements,
    ))
}
