//! A block's string table as read: each text a slice of the block, none longer than OpenStreetMap allows, and a budget
//! on what the block's elements take out of it, as a hostile block may name one text millions of times.

use std::cell::Cell;

use super::wire::{Value, Wire};
use crate::limits::MOST_TEXT;
use crate::{Error, Result};

/// The most texts one table holds, and the most one block's elements may take: each text's bytes and its own 24 (an
/// honest block of 8,000 elements takes a few megabytes).
const MOST_TEXTS: usize = 1 << 20;
const MOST_TAKEN: usize = 64 << 20;

#[derive(Default)]
pub struct Table<'a> {
    texts: Vec<&'a [u8]>,
    taken: Cell<usize>,
}

impl<'a> Table<'a> {
    /// The texts of one StringTable message, after any read before.
    pub fn add(&mut self, table: &'a [u8]) -> Result<()> {
        let mut wire = Wire::new(table);
        while let Some((number, value)) = wire.field()? {
            if let (1, Value::Bytes(text)) = (number, value) {
                if text.len() > MOST_TEXT {
                    return Err(Error::Limit("a text longer than OpenStreetMap allows"));
                }
                if self.texts.len() >= MOST_TEXTS {
                    return Err(Error::Limit("a string table of more than 1,048,576 texts"));
                }
                self.texts.push(text);
            }
        }
        Ok(())
    }

    /// The text at an index, counted against the block's budget.
    pub fn text(&self, index: u64) -> Result<String> {
        let at = usize::try_from(index).map_err(|_| Error::Format("string index"))?;
        let text = self.texts.get(at).ok_or(Error::Format("string index"))?;
        let taken = self.taken.get() + text.len() + size_of::<String>();
        if taken > MOST_TAKEN {
            return Err(Error::Limit("a block's texts past 64 MiB"));
        }
        self.taken.set(taken);
        Ok(String::from_utf8_lossy(text).into_owned())
    }
}
