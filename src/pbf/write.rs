//! Writing a PBF file: its header block, then blocks of at most 8,000 elements of one kind, zlib-compressed, in the
//! order given. Callers keep OpenStreetMap's order (nodes, then ways, then relations, each by id) and say so in the
//! header's optional features ("Sort.Type_then_ID").

use std::io::Write;

use super::encode;
use super::header::{self, Header};
use super::out::Out;
use crate::Result;
use crate::osm::{Element, Kind};

const PER_BLOCK: usize = 8000;

pub struct Writer<W: Write> {
    out: W,
    pending: Vec<Element>,
    kind: Option<Kind>,
}

impl<W: Write> Writer<W> {
    pub fn new(mut out: W, header: &Header) -> Result<Self> {
        blob(&mut out, "OSMHeader", &header::encode(header))?;
        Ok(Writer {
            out,
            pending: Vec::new(),
            kind: None,
        })
    }

    pub fn add(&mut self, element: Element) -> Result<()> {
        if self.kind != Some(element.kind()) || self.pending.len() >= PER_BLOCK {
            self.flush()?;
            self.kind = Some(element.kind());
        }
        self.pending.push(element);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.pending.is_empty() {
            blob(&mut self.out, "OSMData", &encode::block(&self.pending))?;
            self.pending.clear();
        }
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
