//! The graph file's pieces as read, little-endian: numbers, strings (a u32 length, then UTF-8) and lists (a u64 count,
//! then fixed-size records), each list read in one piece; every length is checked against its kind's limit and what
//! is left of the file before room is taken for it.

use std::io::Read;

use crate::{Error, Result};

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

    /// A text of at most `most` bytes.
    pub fn text(&mut self, most: usize) -> Result<String> {
        let length = usize::try_from(self.u32()?).ok();
        let length = length.filter(|length| *length <= most && *length as u64 <= self.left);
        let mut bytes = vec![0; length.ok_or(Error::Format("graph file text length"))?];
        self.fill(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::Format("graph file text"))
    }

    /// A count of items, each taking at least `size` bytes of what is left of the file.
    pub fn count(&mut self, size: u64) -> Result<usize> {
        let count = u64::from_le_bytes(self.raw()?);
        let fits = count
            .checked_mul(size)
            .is_some_and(|bytes| bytes <= self.left);
        let count = usize::try_from(count).ok().filter(|_| fits);
        count.ok_or(Error::Format("graph file list length"))
    }

    /// A list of `N`-byte records, each made by `each`.
    pub fn list<const N: usize, T>(&mut self, each: impl Fn(&[u8; N]) -> T) -> Result<Vec<T>> {
        let size = self.count(N as u64)?.checked_mul(N);
        let size = size.ok_or(Error::Format("graph file list length"))?;
        let mut bytes = Vec::new();
        let full = |_| Error::Limit("a graph file list too large to hold");
        bytes.try_reserve_exact(size).map_err(full)?;
        bytes.resize(size, 0);
        self.fill(&mut bytes)?;
        Ok(bytes
            .chunks_exact(N)
            .map(|chunk| each(chunk.try_into().expect("N bytes")))
            .collect())
    }
}
