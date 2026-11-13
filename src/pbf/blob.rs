//! A PBF file is a run of blobs: a four-byte big-endian length, a BlobHeader naming the blob's kind, and a Blob
//! holding one block, zlib-compressed or raw (<https://wiki.openstreetmap.org/wiki/PBF_Format>).

use std::io::{Read, Seek, SeekFrom};

use super::frame::{MAX_HEADER, blob_header, inflate, read_full};
use crate::{Error, Result};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlobKind {
    /// The file's one header block (OSMHeader).
    Header,
    /// A block of elements (OSMData).
    Data,
    /// A kind this library does not know; readers skip it.
    Other,
}

/// One blob: where it starts in the file, its kind and its block's bytes, inflated.
pub struct Blob {
    pub offset: u64,
    pub kind: BlobKind,
    pub data: Vec<u8>,
}

/// The blobs of a file, in order.
pub struct Blobs<R> {
    input: R,
    offset: u64,
}

impl<R: Read> Blobs<R> {
    pub fn new(input: R) -> Self {
        Blobs { input, offset: 0 }
    }

    /// The next blob, or None at the file's end.
    pub fn next_blob(&mut self) -> Result<Option<Blob>> {
        let start = self.offset;
        let mut size = [0u8; 4];
        match read_full(&mut self.input, &mut size)? {
            0 => return Ok(None),
            4 => {}
            _ => return Err(Error::Format("file ends inside a blob")),
        }
        let header_len = u32::from_be_bytes(size) as usize;
        if header_len > MAX_HEADER {
            return Err(Error::Format("blob header too large"));
        }
        let mut header = vec![0; header_len];
        self.input.read_exact(&mut header)?;
        let (kind, data_len) = blob_header(&header)?;
        let mut blob = vec![0; data_len];
        self.input.read_exact(&mut blob)?;
        self.offset = start + 4 + header_len as u64 + data_len as u64;
        let data = if kind == BlobKind::Other {
            Vec::new()
        } else {
            inflate(&blob)?
        };
        Ok(Some(Blob {
            offset: start,
            kind,
            data,
        }))
    }
}

impl<R: Read + Seek> Blobs<R> {
    /// Continue from a blob found before (its offset).
    pub fn seek(&mut self, offset: u64) -> Result<()> {
        self.input.seek(SeekFrom::Start(offset))?;
        self.offset = offset;
        Ok(())
    }
}
