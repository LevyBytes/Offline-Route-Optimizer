//! One record of each kind as bytes and back: a point as two i32, an edge and a chain as 25 bytes each, an index
//! as a u32.

use crate::graph::{Chain, Edge};
use crate::osm::Point;

pub(super) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

pub(super) fn point(bytes: &[u8; 8]) -> Point {
    Point {
        lat: u32_at(bytes, 0) as i32,
        lon: u32_at(bytes, 4) as i32,
    }
}

pub(super) fn put_point(point: &Point, out: &mut Vec<u8>) {
    out.extend(point.lat.to_le_bytes());
    out.extend(point.lon.to_le_bytes());
}

pub(super) fn put_u32(value: &u32, out: &mut Vec<u8>) {
    out.extend(value.to_le_bytes());
}

pub(super) fn put_edge(e: &Edge, out: &mut Vec<u8>) {
    [e.tail, e.head, e.chain]
        .iter()
        .for_each(|value| put_u32(value, out));
    out.push(u8::from(e.forward));
    [e.time_ms, e.length_dm]
        .iter()
        .for_each(|value| put_u32(value, out));
    out.extend(e.start_bearing.to_le_bytes());
    out.extend(e.end_bearing.to_le_bytes());
}

pub(super) fn edge(b: &[u8; 25]) -> Edge {
    let (tail, head, chain) = (u32_at(b, 0), u32_at(b, 4), u32_at(b, 8));
    let (forward, time_ms, length_dm) = (b[12] != 0, u32_at(b, 13), u32_at(b, 17));
    let (start_bearing, end_bearing) = (
        u16::from_le_bytes([b[21], b[22]]),
        u16::from_le_bytes([b[23], b[24]]),
    );
    Edge {
        tail,
        head,
        chain,
        forward,
        time_ms,
        length_dm,
        start_bearing,
        end_bearing,
    }
}

pub(super) fn put_chain(c: &Chain, out: &mut Vec<u8>) {
    [c.first, c.count, c.name]
        .iter()
        .for_each(|value| put_u32(value, out));
    out.push(c.class);
    [c.length_dm, c.forward, c.backward]
        .iter()
        .for_each(|value| put_u32(value, out));
}

pub(super) fn chain(b: &[u8; 25]) -> Chain {
    let (first, count, name, class) = (u32_at(b, 0), u32_at(b, 4), u32_at(b, 8), b[12]);
    let (length_dm, forward, backward) = (u32_at(b, 13), u32_at(b, 17), u32_at(b, 21));
    Chain {
        first,
        count,
        name,
        class,
        length_dm,
        forward,
        backward,
    }
}

pub(super) fn u32_record(bytes: &[u8; 4]) -> u32 {
    u32_at(bytes, 0)
}
