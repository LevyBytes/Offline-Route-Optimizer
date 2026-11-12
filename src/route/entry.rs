//! The search's heap entries: an edge reached at its head, or a spot reached, each with its cost and leg so far.

use std::cmp::Ordering;

use super::Leg;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Edge(u32),
    Spot(u32),
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
    /// Reversed, so the heap gives the least cost first (then the lower item, so a search is always the same).
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .key
            .total_cmp(&self.key)
            .then_with(|| other.item.cmp(&self.item))
    }
}
