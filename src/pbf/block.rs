//! An OSMData block (PrimitiveBlock): a string table, the scale of its coordinates, and groups that each hold nodes,
//! dense nodes, ways or relations.

use super::kinds::Kinds;
use super::wire::{Value, Wire};
use super::{dense, items};
use crate::osm::{Element, Point};
use crate::{Error, Result};

pub struct Block<'a> {
    pub(super) strings: Vec<&'a [u8]>,
    granularity: i64,
    lat_offset: i64,
    lon_offset: i64,
    pub(super) groups: Vec<&'a [u8]>,
}

impl<'a> Block<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let mut block = Block {
            strings: Vec::new(),
            granularity: 100,
            lat_offset: 0,
            lon_offset: 0,
            groups: Vec::new(),
        };
        let mut wire = Wire::new(data);
        while let Some((number, value)) = wire.field()? {
            match (number, value) {
                (1, Value::Bytes(table)) => {
                    let mut strings = Wire::new(table);
                    while let Some((number, value)) = strings.field()? {
                        if let (1, Value::Bytes(text)) = (number, value) {
                            block.strings.push(text);
                        }
                    }
                }
                (2, Value::Bytes(group)) => block.groups.push(group),
                (17, Value::Int(value)) => block.granularity = value as i64,
                (19, Value::Int(value)) => block.lat_offset = value as i64,
                (20, Value::Int(value)) => block.lon_offset = value as i64,
                _ => {}
            }
        }
        Ok(block)
    }

    pub(super) fn string(&self, index: u64) -> Result<String> {
        let bytes = self
            .strings
            .get(index as usize)
            .ok_or(Error::Format("string index"))?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    /// A position from the block's raw numbers: billionths of a degree, then rounded to ten-millionths.
    pub(super) fn point(&self, lat: i64, lon: i64) -> Point {
        let e7 = |offset: i64, raw: i64| {
            let nano = offset + self.granularity * raw;
            (nano.div_euclid(100) + i64::from(nano.rem_euclid(100) >= 50)) as i32
        };
        Point {
            lat: e7(self.lat_offset, lat),
            lon: e7(self.lon_offset, lon),
        }
    }

    /// Every element of the wanted kinds, with its tags, in the block's order.
    pub fn elements(&self, want: Kinds, out: &mut Vec<Element>) -> Result<()> {
        for group in &self.groups {
            let mut wire = Wire::new(group);
            while let Some((number, value)) = wire.field()? {
                let Value::Bytes(bytes) = value else { continue };
                match number {
                    1 if want.nodes => out.push(Element::Node(items::node(self, bytes)?)),
                    2 if want.nodes => dense::nodes(self, bytes, out)?,
                    3 if want.ways => out.push(Element::Way(items::way(self, bytes)?)),
                    4 if want.relations => {
                        out.push(Element::Relation(items::relation(self, bytes)?))
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
