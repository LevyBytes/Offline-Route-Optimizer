//! The graph file's pieces as written, little-endian: numbers, strings (a u32 length, then UTF-8) and lists (a u64
//! count, then fixed-size records), each text within the limit its reader keeps.

use std::io::Write;

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

    /// A text, refused past `most` bytes (as `Take::text` would refuse it).
    pub fn text(&mut self, text: &str, most: usize) -> Result<()> {
        if text.len() > most {
            return Err(Error::Limit("a graph text longer than its kind allows"));
        }
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
