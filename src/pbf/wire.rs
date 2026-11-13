//! Protocol Buffers' wire format, the part PBF files use: varints, zigzag numbers, fixed widths, length-delimited
//! fields and packed repeated numbers (<https://protobuf.dev/programming-guides/encoding/>).

use crate::{Error, Result};

/// A reader over one message's bytes, field by field.
#[derive(Clone)]
pub struct Wire<'a> {
    data: &'a [u8],
    pub(super) at: usize,
}

/// A field's value by its wire type (fixed-width values, which PBF files do not use, are skipped over).
pub enum Value<'a> {
    Int(u64),
    Bytes(&'a [u8]),
    Fixed,
}

impl<'a> Wire<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Wire { data, at: 0 }
    }

    pub fn varint(&mut self) -> Result<u64> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = *self
                .data
                .get(self.at)
                .ok_or(Error::Format("varint past the end"))?;
            self.at += 1;
            // The tenth byte holds only the 64th bit.
            if shift == 63 && byte > 1 {
                return Err(Error::Format("varint beyond 64 bits"));
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                return Ok(value);
            }
        }
        Err(Error::Format("varint longer than ten bytes"))
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.data.len());
        let end = end.ok_or(Error::Format("field past the end of its message"))?;
        let bytes = &self.data[self.at..end];
        self.at = end;
        Ok(bytes)
    }

    /// The next field's number and value, or None at the message's end.
    pub fn field(&mut self) -> Result<Option<(u32, Value<'a>)>> {
        if self.at >= self.data.len() {
            return Ok(None);
        }
        let key = self.varint()?;
        let number = u32::try_from(key >> 3).map_err(|_| Error::Format("field number"))?;
        let value = match key & 7 {
            0 => Value::Int(self.varint()?),
            1 => {
                self.take(8)?;
                Value::Fixed
            }
            2 => {
                let count =
                    usize::try_from(self.varint()?).map_err(|_| Error::Format("field length"))?;
                Value::Bytes(self.take(count)?)
            }
            5 => {
                self.take(4)?;
                Value::Fixed
            }
            _ => return Err(Error::Format("unknown wire type")),
        };
        Ok(Some((number, value)))
    }
}
