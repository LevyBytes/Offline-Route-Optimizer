//! A kept extract brought up to date: its elements read in their order (nodes, then ways, then relations, each by
//! id, as Geofabrik writes them), each replaced by its change, left out where deleted, and new ones put in their
//! place, written to a part file beside `new` and renamed over it. The new header takes the state the changes reach;
//! a state older than the extract's own is refused, as its changes would put older versions back (the same state
//! again changes nothing). That the files follow one another without a gap is the program's to see to.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use super::State;
use super::osc::Changes;
use crate::osm::Kind;
use crate::pbf::{BlobKind, Blobs, Block, Kinds, Writer, decode_header};
use crate::temp::Part;
use crate::{Error, Result};

/// Where a merge reports its progress: bytes of the old file read of its size.
pub type Progress<'a> = dyn FnMut(u64, u64) + 'a;

type Out = (Writer<BufWriter<File>>, Part);

/// The new file begun from the old header, once the state is known to be no older than the extract's.
fn begin(data: &[u8], state: State, new: &Path) -> Result<Out> {
    let mut header = decode_header(data)?;
    let older = |kept: Option<i64>, given: i64| kept.is_some_and(|kept| given < kept);
    if older(header.replication_sequence, state.sequence)
        || older(header.replication_time, state.time)
    {
        return Err(Error::Input("a state older than the extract's"));
    }
    header.replication_time = Some(state.time);
    header.replication_sequence = Some(state.sequence);
    let (part, file) = Part::new(new)?;
    let writer = Writer::new(BufWriter::with_capacity(1 << 20, file), &header)?;
    Ok((writer, part))
}

pub fn merge(
    old: &Path,
    changes: Changes,
    state: State,
    new: &Path,
    progress: &mut Progress<'_>,
) -> Result<()> {
    let file = File::open(old)?;
    let size = file.metadata()?.len().max(1);
    let mut blobs = Blobs::new(BufReader::with_capacity(1 << 20, file));
    let (mut out, mut pending, mut elements) = (None, changes.into_iter().peekable(), Vec::new());
    let mut last: Option<(Kind, i64)> = None;
    while let Some(blob) = blobs.next_blob()? {
        progress(blob.offset, size);
        match blob.kind {
            BlobKind::Header if out.is_some() => {
                return Err(Error::Format("a second header block"));
            }
            BlobKind::Header => out = Some(begin(&blob.data, state, new)?),
            BlobKind::Data => {
                let (writer, _) = out
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
    let (mut writer, part) = out.ok_or(Error::Format("file without a header block"))?;
    for change in pending.filter_map(|(_, change)| change) {
        writer.add(change)?;
    }
    let file = writer
        .finish()?
        .into_inner()
        .map_err(|error| error.into_error())?;
    file.sync_all()?;
    drop((file, blobs));
    part.finish(new)?;
    progress(size, size);
    Ok(())
}
