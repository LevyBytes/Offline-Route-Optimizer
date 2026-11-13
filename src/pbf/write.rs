//! Writing a PBF file: its header block, then blocks of at most 8,000 elements of one kind, zlib-compressed, in the
//! order given. Callers keep OpenStreetMap's order (nodes, then ways, then relations, each by id) and say so in the
//! header's optional features ("Sort.Type_then_ID"). A block also ends past about 8 MiB of elements (counted high, each
//! text as if new), and one element may take at most 16 MiB, so every block stays inside the 32 MiB a reader takes.

use std::io::Write;

use super::encode;
use super::header::{self, Header};
use super::out::Out;
use crate::osm::{Element, Kind, Tags};
use crate::{Error, Result};

const PER_BLOCK: usize = 8000;
const BLOCK_BYTES: usize = 8 << 20;
const MOST_ELEMENT: usize = 16 << 20;

pub struct Writer<W: Write> {
    out: W,
    pending: Vec<Element>,
    kind: Option<Kind>,
    bytes: usize,
}

/// At least what an element takes in a block: its numbers at ten bytes each, its texts and their places in the table.
fn size(element: &Element) -> usize {
    let tags = |tags: &Tags| {
        tags.iter()
            .map(|(k, v)| k.len() + v.len() + 20)
            .sum::<usize>()
    };
    match element {
        Element::Node(node) => 40 + tags(&node.tags),
        Element::Way(way) => 20 + 10 * way.refs.len() + tags(&way.tags),
        Element::Relation(relation) => {
            let members = relation.members.iter().map(|member| 30 + member.role.len());
            20 + members.sum::<usize>() + tags(&relation.tags)
        }
    }
}

impl<W: Write> Writer<W> {
    pub fn new(mut out: W, header: &Header) -> Result<Self> {
        blob(&mut out, "OSMHeader", &header::encode(header))?;
        Ok(Writer {
            out,
            pending: Vec::new(),
            kind: None,
            bytes: 0,
        })
    }

    pub fn add(&mut self, element: Element) -> Result<()> {
        let size = size(&element);
        if size > MOST_ELEMENT {
            return Err(Error::Limit("an element too large for one block"));
        }
        let full = self.pending.len() >= PER_BLOCK || self.bytes + size > BLOCK_BYTES;
        if self.kind != Some(element.kind()) || full {
            self.flush()?;
            self.kind = Some(element.kind());
        }
        self.bytes += size;
        self.pending.push(element);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.pending.is_empty() {
            blob(&mut self.out, "OSMData", &encode::block(&self.pending))?;
            self.pending.clear();
        }
        self.bytes = 0;
        Ok(())
    }

    /// The last block written and the output flushed.
    pub fn finish(mut self) -> Result<W> {
        self.flush()?;
        self.out.flush()?;
        Ok(self.out)
    }
}

fn blob(out: &mut impl Write, kind: &str, data: &[u8]) -> Result<()> {
    let zlib = miniz_oxide::deflate::compress_to_vec_zlib(data, 6);
    let mut blob = Out::default();
    blob.int(2, data.len() as u64);
    blob.bytes(3, &zlib);
    let mut head = Out::default();
    head.bytes(1, kind.as_bytes());
    head.int(3, blob.bytes.len() as u64);
    out.write_all(&(head.bytes.len() as u32).to_be_bytes())?;
    out.write_all(&head.bytes)?;
    out.write_all(&blob.bytes)?;
    Ok(())
}
