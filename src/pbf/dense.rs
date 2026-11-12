//! DenseNodes: a group's nodes as parallel lists of ids, latitudes and longitudes (each a difference from the one
//! before) and one list of tag string indexes, each node's pairs ended by a zero.

use super::block::Block;
use super::numbers::{deltas, packed};
use super::wire::{Value, Wire};
use crate::osm::{Element, Node, Point};
use crate::{Error, Result};

struct Dense {
    ids: Vec<i64>,
    lats: Vec<i64>,
    lons: Vec<i64>,
    keys_vals: Vec<u64>,
}

fn read(data: &[u8], tags: bool) -> Result<Dense> {
    let mut dense = Dense {
        ids: Vec::new(),
        lats: Vec::new(),
        lons: Vec::new(),
        keys_vals: Vec::new(),
    };
    let mut wire = Wire::new(data);
    while let Some((number, value)) = wire.field()? {
        let Value::Bytes(bytes) = value else { continue };
        match number {
            1 => dense.ids = deltas(bytes)?,
            8 => dense.lats = deltas(bytes)?,
            9 => dense.lons = deltas(bytes)?,
            10 if tags => dense.keys_vals = packed(bytes)?,
            _ => {}
        }
    }
    if dense.lats.len() != dense.ids.len() || dense.lons.len() != dense.ids.len() {
        return Err(Error::Format("dense nodes' lists differ in length"));
    }
    Ok(dense)
}

pub(super) fn positions(
    block: &Block,
    data: &[u8],
    found: &mut impl FnMut(i64, Point),
) -> Result<()> {
    let dense = read(data, false)?;
    for index in 0..dense.ids.len() {
        found(
            dense.ids[index],
            block.point(dense.lats[index], dense.lons[index]),
        );
    }
    Ok(())
}

pub(super) fn nodes(block: &Block, data: &[u8], out: &mut Vec<Element>) -> Result<()> {
    let dense = read(data, true)?;
    let mut pairs = dense.keys_vals.iter().copied();
    for index in 0..dense.ids.len() {
        let mut tags = Vec::new();
        // Without any tags in the block the list is empty; else each node's pairs end with a zero.
        while !dense.keys_vals.is_empty() {
            let key = pairs.next().ok_or(Error::Format("dense tags end early"))?;
            if key == 0 {
                break;
            }
            let value = pairs
                .next()
                .ok_or(Error::Format("dense tag without a value"))?;
            tags.push((block.string(key)?, block.string(value)?));
        }
        let at = block.point(dense.lats[index], dense.lons[index]);
        out.push(Element::Node(Node {
            id: dense.ids[index],
            at,
            tags,
        }));
    }
    Ok(())
}
