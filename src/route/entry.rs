//! The search's heap entries: an edge reached at its head, or a spot reached (by a table's search, or by a drive's,
//! which keeps the way it came), each with its cost and leg so far.

use std::cmp::Ordering;

use super::Leg;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Edge(u32),
    Spot(u32),
    /// A drive's spot reached: an index into the search's `vias`, the way it came.
    Via(u32),
}

#[derive(Clone, Copy)]
pub struct Entry {
    pub key: f64,
    pub leg: Leg,
    pub item: Item,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    /// Reversed, so the heap gives the least cost first; at an equal cost the shorter time, then the shorter length,
    /// then the lower item. So a search is always the same, a spot's leg never depends on what else the search looks
    /// for, and a drive's leg is the one its table finds.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .key
            .total_cmp(&self.key)
            .then_with(|| other.leg.seconds.total_cmp(&self.leg.seconds))
            .then_with(|| other.leg.meters.total_cmp(&self.leg.meters))
            .then_with(|| other.item.cmp(&self.item))
    }
}
