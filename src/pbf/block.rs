//! An OSMData block (PrimitiveBlock): a string table, the scale of its coordinates, and groups that each hold nodes,
//! dense nodes, ways or relations.

use super::kinds::Kinds;
use super::table::Table;
use super::wire::{Value, Wire};
use super::{dense, items};
use crate::osm::{Element, Point};
use crate::{Error, Result};

pub struct Block<'a> {
    table: Table<'a>,
    granularity: i64,
    lat_offset: i64,
    lon_offset: i64,
    pub(super) groups: Vec<&'a [u8]>,
}

impl<'a> Block<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let (table, groups) = (Table::default(), Vec::new());
        let (granularity, lat_offset, lon_offset) = (100, 0, 0);
        let mut block = Block {
            table,
            granularity,
            lat_offset,
            lon_offset,
            groups,
        };
        let mut wire = Wire::new(data);
        while let Some((number, value)) = wire.field()? {
            match (number, value) {
                (1, Value::Bytes(table)) => block.table.add(table)?,
                (2, Value::Bytes(group)) => block.groups.push(group),
                (17, Value::Int(value)) => block.granularity = value as i64,
                (19, Value::Int(value)) => block.lat_offset = value as i64,
                (20, Value::Int(value)) => block.lon_offset = value as i64,
                _ => {}
            }
        }
        if block.granularity <= 0 {
            return Err(Error::Format("granularity"));
        }
        Ok(block)
    }

    pub(super) fn string(&self, index: u64) -> Result<String> {
        self.table.text(index)
    }

    /// A position from the block's raw numbers: billionths of a degree, then rounded to ten-millionths. Numbers that
    /// overflow are refused; a position off the earth is kept as it is, for the build to leave out (Point::valid).
    pub(super) fn point(&self, lat: i64, lon: i64) -> Result<Point> {
        let e7 = |offset: i64, raw: i64| {
            let nano = self
                .granularity
                .checked_mul(raw)
                .and_then(|n| n.checked_add(offset));
            let e7 = nano.map(|nano| nano.div_euclid(100) + i64::from(nano.rem_euclid(100) >= 50));
            e7.and_then(|e7| i32::try_from(e7).ok())
                .ok_or(Error::Format("node position"))
        };
        Ok(Point {
            lat: e7(self.lat_offset, lat)?,
            lon: e7(self.lon_offset, lon)?,
        })
    }

    /// Every element of the wanted kinds, with its tags, in the block's order (an element past OpenStreetMap's own
    /// limits on a way's points and a relation's members is left out: items.rs).
    pub fn elements(&self, want: Kinds, out: &mut Vec<Element>) -> Result<()> {
        for group in &self.groups {
            let mut wire = Wire::new(group);
            while let Some((number, value)) = wire.field()? {
                let Value::Bytes(bytes) = value else { continue };
                match number {
                    1 if want.nodes => out.push(Element::Node(items::node(self, bytes)?)),
                    2 if want.nodes => dense::nodes(self, bytes, out)?,
                    3 if want.ways => out.extend(items::way(self, bytes)?.map(Element::Way)),
                    4 if want.relations => {
                        out.extend(items::relation(self, bytes)?.map(Element::Relation))
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
