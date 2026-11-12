//! A block read quickly: which kinds of element it holds, and its nodes' positions without their tags.

use super::block::Block;
use super::dense;
use super::items;
use super::kinds::Kinds;
use super::wire::{Value, Wire};
use crate::Result;
use crate::osm::Point;

impl Block<'_> {
    /// The kinds of element the block holds.
    pub fn kinds(&self) -> Result<Kinds> {
        let mut kinds = Kinds::default();
        for group in &self.groups {
            let mut wire = Wire::new(group);
            while let Some((number, _)) = wire.field()? {
                match number {
                    1 | 2 => kinds.nodes = true,
                    3 => kinds.ways = true,
                    4 => kinds.relations = true,
                    _ => {}
                }
            }
        }
        Ok(kinds)
    }

    /// Each node's id and position, without its tags: the fast way through a file's nodes.
    pub fn positions(&self, mut found: impl FnMut(i64, Point)) -> Result<()> {
        for group in &self.groups {
            let mut wire = Wire::new(group);
            while let Some((number, value)) = wire.field()? {
                match (number, value) {
                    (1, Value::Bytes(bytes)) => {
                        let node = items::node(self, bytes)?;
                        found(node.id, node.at);
                    }
                    (2, Value::Bytes(bytes)) => dense::positions(self, bytes, &mut found)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
