//! The graph file's pieces, little-endian: numbers, strings (a u32 length, then UTF-8) and lists (a u64 count, then
//! fixed-size records), each list read in one piece and never longer than what is left of the file.

use std::io::{Read, Write};

use crate::{Error, Result};

pub struct Put<W: Write> {
    pub out: W,
    record: Vec<u8>,
}

impl<W: Write> Put<W> {
    pub fn new(out: W) -> Self {
        Put {
            out,
            record: Vec::new(),
        }
    }

    pub fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        Ok(self.out.write_all(bytes)?)
    }

    pub fn text(&mut self, text: &str) -> Result<()> {
        self.raw(&(text.len() as u32).to_le_bytes())?;
        self.raw(text.as_bytes())
    }

    /// A list of records, each written by `each` into the buffer it is given.
    pub fn list<T>(&mut self, items: &[T], mut each: impl FnMut(&T, &mut Vec<u8>)) -> Result<()> {
        self.raw(&(items.len() as u64).to_le_bytes())?;
        for chunk in items.chunks(65_536) {
            self.record.clear();
            chunk.iter().for_each(|item| each(item, &mut self.record));
            self.out.write_all(&self.record)?;
        }
        Ok(())
    }
}

pub struct Take<R: Read> {
    input: R,
    left: u64,
}

impl<R: Read> Take<R> {
    pub fn new(input: R, size: u64) -> Self {
        Take { input, left: size }
    }

    pub fn raw<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut bytes = [0; N];
        self.fill(&mut bytes)?;
        Ok(bytes)
    }

    fn fill(&mut self, bytes: &mut [u8]) -> Result<()> {
        if bytes.len() as u64 > self.left {
            return Err(Error::Format("graph file ends early"));
        }
        self.input.read_exact(bytes)?;
        self.left -= bytes.len() as u64;
        Ok(())
    }

    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.raw()?))
    }

    pub fn i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.raw()?))
    }

    pub fn text(&mut self) -> Result<String> {
        let mut bytes = vec![0; self.u32()? as usize];
        self.fill(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::Format("graph file text"))
    }

    /// A list of `N`-byte records, each made by `each`.
    pub fn list<const N: usize, T>(&mut self, each: impl Fn(&[u8; N]) -> T) -> Result<Vec<T>> {
        let count = u64::from_le_bytes(self.raw()?);
        let size = count
            .checked_mul(N as u64)
            .filter(|size| *size <= self.left);
        let mut bytes = vec![0; size.ok_or(Error::Format("graph file list length"))? as usize];
        self.fill(&mut bytes)?;
        Ok(bytes
            .chunks_exact(N)
            .map(|chunk| each(chunk.try_into().expect("N bytes")))
            .collect())
    }
}
