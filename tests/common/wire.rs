//! Protocol Buffers' wire format written by hand, for blocks and headers no honest writer would make.

pub fn varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(value as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

pub fn int(number: u32, value: u64, out: &mut Vec<u8>) {
    varint(u64::from(number) << 3, out);
    varint(value, out);
}

pub fn bytes(number: u32, data: &[u8], out: &mut Vec<u8>) {
    varint(u64::from(number) << 3 | 2, out);
    varint(data.len() as u64, out);
    out.extend_from_slice(data);
}

pub fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}
