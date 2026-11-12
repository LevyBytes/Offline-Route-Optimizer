//! Writing Protocol Buffers' wire format: the mirror of wire.rs, for the files this library writes.

/// A message being written.
#[derive(Default)]
pub struct Out {
    pub bytes: Vec<u8>,
}

impl Out {
    pub fn varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.bytes.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.bytes.push(value as u8);
    }

    fn key(&mut self, number: u32, wire: u64) {
        self.varint((u64::from(number) << 3) | wire);
    }

    /// A varint field (int32, int64, uint32, enum, bool).
    pub fn int(&mut self, number: u32, value: u64) {
        self.key(number, 0);
        self.varint(value);
    }

    /// A zigzag field (sint32, sint64).
    pub fn sint(&mut self, number: u32, value: i64) {
        self.int(number, zag(value));
    }

    /// A length-delimited field (bytes, string, a nested message, a packed list).
    pub fn bytes(&mut self, number: u32, value: &[u8]) {
        self.key(number, 2);
        self.varint(value.len() as u64);
        self.bytes.extend_from_slice(value);
    }

    /// A packed list of varints, left out when empty.
    pub fn packed(&mut self, number: u32, values: impl IntoIterator<Item = u64>) {
        let mut list = Out::default();
        values.into_iter().for_each(|value| list.varint(value));
        if !list.bytes.is_empty() {
            self.bytes(number, &list.bytes);
        }
    }

    /// A packed sint64 list stored as differences from the value before.
    pub fn deltas(&mut self, number: u32, values: impl IntoIterator<Item = i64>) {
        let mut last = 0i64;
        self.packed(
            number,
            values.into_iter().map(|value| {
                let delta = value.wrapping_sub(last);
                last = value;
                zag(delta)
            }),
        );
    }
}

/// Zigzag encoding of a signed number.
pub fn zag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}
