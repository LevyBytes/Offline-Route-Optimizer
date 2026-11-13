//! The OSMHeader block: the file's bounds, the features a reader needs, and the replication state that says which
//! change files bring it up to date (Geofabrik writes the base URL and sequence number there).

use super::numbers::zigzag;
use super::out::Out;
use super::wire::{Value, Wire};
use crate::{Error, Result};

/// The features this reader has (a file that requires another, history for one, is refused, as the format asks) and
/// the most feature names a header may list (an honest one lists a handful).
const HAS: [&str; 2] = ["OsmSchema-V0.6", "DenseNodes"];
const MOST_FEATURES: usize = 64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Header {
    /// Left, right, top and bottom in billionths of a degree.
    pub bbox: Option<[i64; 4]>,
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub program: String,
    pub source: String,
    /// Seconds since 1970 (UTC) of the data's last change.
    pub replication_time: Option<i64>,
    pub replication_sequence: Option<i64>,
    /// Where the file's change files are (e.g. <https://download.geofabrik.de/north-america/us/nevada-updates>).
    pub replication_url: Option<String>,
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn decode(data: &[u8]) -> Result<Header> {
    let mut header = Header::default();
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Bytes(bbox)) => header.bbox = Some(decode_bbox(bbox)?),
            (4, Value::Bytes(name)) => header.required.push(text(name)),
            (5, Value::Bytes(name)) => header.optional.push(text(name)),
            (16, Value::Bytes(name)) => header.program = text(name),
            (17, Value::Bytes(name)) => header.source = text(name),
            (32, Value::Int(time)) => header.replication_time = Some(time as i64),
            (33, Value::Int(sequence)) => header.replication_sequence = Some(sequence as i64),
            (34, Value::Bytes(url)) => header.replication_url = Some(text(url)),
            _ => {}
        }
        if header.required.len() + header.optional.len() > MOST_FEATURES {
            return Err(Error::Limit("a header naming more than 64 features"));
        }
    }
    let has = |name: &String| HAS.contains(&name.as_str());
    if !header.required.iter().all(has) {
        return Err(Error::Format("requires a feature this reader lacks"));
    }
    Ok(header)
}

fn decode_bbox(data: &[u8]) -> Result<[i64; 4]> {
    let mut bbox = [0i64; 4];
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        if let (1..=4, Value::Int(raw)) = (number, value) {
            bbox[number as usize - 1] = zigzag(raw);
        }
    }
    Ok(bbox)
}

pub fn encode(header: &Header) -> Vec<u8> {
    let mut out = Out::default();
    if let Some(bbox) = header.bbox {
        let mut inner = Out::default();
        for (index, value) in bbox.iter().enumerate() {
            inner.sint(index as u32 + 1, *value);
        }
        out.bytes(1, &inner.bytes);
    }
    let names = header.required.iter().map(|name| (4, name));
    let names = names.chain(header.optional.iter().map(|name| (5, name)));
    for (number, text) in names.chain([(16, &header.program), (17, &header.source)]) {
        if !text.is_empty() || number < 16 {
            out.bytes(number, text.as_bytes());
        }
    }
    if let Some(time) = header.replication_time {
        out.int(32, time as u64);
    }
    if let Some(sequence) = header.replication_sequence {
        out.int(33, sequence as u64);
    }
    if let Some(url) = &header.replication_url {
        out.bytes(34, url.as_bytes());
    }
    out.bytes
}
