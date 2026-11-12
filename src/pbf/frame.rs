//! A blob's framing: its header (kind and size) and its block, raw or zlib-compressed, inflated within the format's
//! limits (a header of at most 64 KiB, a block of at most 32 MiB).

use std::io::{ErrorKind, Read};

use super::blob::BlobKind;
use super::wire::{Value, Wire};
use crate::{Error, Result};

pub const MAX_HEADER: usize = 64 * 1024;
pub const MAX_BLOCK: usize = 32 * 1024 * 1024;

pub fn read_full(input: &mut impl Read, buffer: &mut [u8]) -> Result<usize> {
    let mut done = 0;
    while done < buffer.len() {
        match input.read(&mut buffer[done..]) {
            Ok(0) => break,
            Ok(count) => done += count,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(done)
}

pub fn blob_header(bytes: &[u8]) -> Result<(BlobKind, usize)> {
    let (mut kind, mut size) = (None, None);
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Bytes(name)) => {
                kind = Some(match name {
                    b"OSMHeader" => BlobKind::Header,
                    b"OSMData" => BlobKind::Data,
                    _ => BlobKind::Other,
                })
            }
            (3, Value::Int(value)) => {
                size = usize::try_from(value)
                    .ok()
                    .filter(|size| *size <= MAX_BLOCK)
            }
            _ => {}
        }
    }
    Ok((
        kind.ok_or(Error::Format("blob without a type"))?,
        size.ok_or(Error::Format("blob size"))?,
    ))
}

pub fn inflate(blob: &[u8]) -> Result<Vec<u8>> {
    let (mut raw, mut raw_size, mut zlib) = (None, None, None);
    let mut wire = Wire::new(blob);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Bytes(bytes)) => raw = Some(bytes),
            (2, Value::Int(size)) => raw_size = usize::try_from(size).ok(),
            (3, Value::Bytes(bytes)) => zlib = Some(bytes),
            (4..=7, _) => return Err(Error::Format("block compression other than zlib")),
            _ => {}
        }
    }
    if let Some(bytes) = raw {
        return Ok(bytes.to_vec());
    }
    let (zlib, size) = (
        zlib.ok_or(Error::Format("blob without data"))?,
        raw_size.unwrap_or(MAX_BLOCK),
    );
    let data = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(zlib, size.min(MAX_BLOCK))
        .map_err(|_| Error::Format("zlib data"))?;
    if raw_size.is_some_and(|size| size != data.len()) {
        return Err(Error::Format("inflated block size"));
    }
    Ok(data)
}
