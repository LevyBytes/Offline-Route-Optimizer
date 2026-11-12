//! A block's single nodes, ways and relations: tags as two lists of string indexes, references and member ids as
//! differences from the one before.

use super::block::Block;
use super::numbers::{deltas, packed, zigzag};
use super::wire::{Value, Wire};
use crate::osm::{Kind, Member, Node, Relation, Tags, Way};
use crate::{Error, Result};

fn tags(block: &Block, keys: &[u64], vals: &[u64]) -> Result<Tags> {
    if keys.len() != vals.len() {
        return Err(Error::Format("tag keys and values differ in number"));
    }
    keys.iter()
        .zip(vals)
        .map(|(k, v)| Ok((block.string(*k)?, block.string(*v)?)))
        .collect()
}

pub(super) fn node(block: &Block, data: &[u8]) -> Result<Node> {
    let (mut id, mut keys, mut vals, mut lat, mut lon) = (0, Vec::new(), Vec::new(), 0, 0);
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Int(raw)) => id = zigzag(raw),
            (2, Value::Bytes(bytes)) => keys = packed(bytes)?,
            (3, Value::Bytes(bytes)) => vals = packed(bytes)?,
            (8, Value::Int(raw)) => lat = zigzag(raw),
            (9, Value::Int(raw)) => lon = zigzag(raw),
            _ => {}
        }
    }
    Ok(Node {
        id,
        at: block.point(lat, lon),
        tags: tags(block, &keys, &vals)?,
    })
}

pub(super) fn way(block: &Block, data: &[u8]) -> Result<Way> {
    let (mut id, mut keys, mut vals, mut refs) = (0, Vec::new(), Vec::new(), Vec::new());
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Int(raw)) => id = raw as i64,
            (2, Value::Bytes(bytes)) => keys = packed(bytes)?,
            (3, Value::Bytes(bytes)) => vals = packed(bytes)?,
            (8, Value::Bytes(bytes)) => refs = deltas(bytes)?,
            _ => {}
        }
    }
    Ok(Way {
        id,
        refs,
        tags: tags(block, &keys, &vals)?,
    })
}

pub(super) fn relation(block: &Block, data: &[u8]) -> Result<Relation> {
    let (mut id, mut keys, mut vals) = (0, Vec::new(), Vec::new());
    let (mut roles, mut ids, mut types) = (Vec::new(), Vec::new(), Vec::new());
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        match (number, value) {
            (1, Value::Int(raw)) => id = raw as i64,
            (2, Value::Bytes(bytes)) => keys = packed(bytes)?,
            (3, Value::Bytes(bytes)) => vals = packed(bytes)?,
            (8, Value::Bytes(bytes)) => roles = packed(bytes)?,
            (9, Value::Bytes(bytes)) => ids = deltas(bytes)?,
            (10, Value::Bytes(bytes)) => types = packed(bytes)?,
            _ => {}
        }
    }
    if roles.len() != ids.len() || types.len() != ids.len() {
        return Err(Error::Format("relation member lists differ in length"));
    }
    let members = (0..ids.len()).map(|index| {
        let kind = match types[index] {
            0 => Kind::Node,
            1 => Kind::Way,
            2 => Kind::Relation,
            _ => return Err(Error::Format("member type")),
        };
        Ok(Member {
            kind,
            id: ids[index],
            role: block.string(roles[index])?,
        })
    });
    Ok(Relation {
        id,
        members: members.collect::<Result<_>>()?,
        tags: tags(block, &keys, &vals)?,
    })
}
