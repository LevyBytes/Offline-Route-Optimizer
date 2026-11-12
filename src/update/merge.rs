//! A kept extract brought up to date: its elements read in their order (nodes, then ways, then relations, each by
//! id, as Geofabrik writes them), each replaced by its change, left out where deleted, and new ones put in their
//! place, written to a new file through `path` with the extension "part" and renamed over it. The new header keeps
//! the old one's and takes the state the changes reach.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use super::State;
use super::osc::Changes;
use crate::osm::Kind;
use crate::pbf::{BlobKind, Blobs, Block, Header, Kinds, Writer, decode_header};
use crate::{Error, Result};

/// Where a merge reports its progress: bytes of the old file read of its size.
pub type Progress<'a> = dyn FnMut(u64, u64) + 'a;

pub fn merge(
    old: &Path,
    changes: Changes,
    state: State,
    new: &Path,
    progress: &mut Progress<'_>,
) -> Result<()> {
    let size = fs::metadata(old)?.len().max(1);
    let mut blobs = Blobs::new(BufReader::with_capacity(1 << 20, File::open(old)?));
    let (mut writer, mut pending, mut elements) =
        (None, changes.into_iter().peekable(), Vec::new());
    let mut last: Option<(Kind, i64)> = None;
    while let Some(blob) = blobs.next_blob()? {
        progress(blob.offset, size);
        match blob.kind {
            BlobKind::Header => {
                let mut header: Header = decode_header(&blob.data)?;
                header.replication_time = Some(state.time);
                header.replication_sequence = Some(state.sequence);
                let part = new.with_extension("part");
                let out = BufWriter::with_capacity(1 << 20, File::create(&part)?);
                writer = Some(Writer::new(out, &header)?);
            }
            BlobKind::Data => {
                let writer = writer
                    .as_mut()
                    .ok_or(Error::Format("data before the header block"))?;
                elements.clear();
                Block::parse(&blob.data)?.elements(Kinds::ALL, &mut elements)?;
                for element in elements.drain(..) {
                    let key = (element.kind(), element.id());
                    if last.is_some_and(|last| key <= last) {
                        return Err(Error::Format("extract not sorted by kind and id"));
                    }
                    last = Some(key);
                    let mut replaced = false;
                    while let Some((change_key, change)) = pending.next_if(|(k, _)| *k <= key) {
                        replaced |= change_key == key;
                        if let Some(change) = change {
                            writer.add(change)?;
                        }
                    }
                    if !replaced {
                        writer.add(element)?;
                    }
                }
            }
            BlobKind::Other => {}
        }
    }
    let mut writer = writer.ok_or(Error::Format("file without a header block"))?;
    for change in pending.filter_map(|(_, change)| change) {
        writer.add(change)?;
    }
    let file = writer
        .finish()?
        .into_inner()
        .map_err(|error| error.into_error())?;
    file.sync_all()?;
    drop((file, blobs));
    fs::rename(new.with_extension("part"), new)?;
    progress(size, size);
    Ok(())
}
