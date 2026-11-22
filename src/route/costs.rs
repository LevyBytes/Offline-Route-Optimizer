//! What a search pays: the cost and leg of driving part of an edge, keeping an edge reached more cheaply, a turn's
//! time from the table and the spots looked for on a chain.

use std::ops::Range;

use super::entry::{Entry, Item};
use super::search::Search;
use super::{Goal, Leg};

impl Search<'_> {
    /// A cost and leg after driving `part` of an edge.
    pub(super) fn along(&self, (key, leg): (f64, Leg), edge: u32, part: f64) -> (f64, Leg) {
        let edge = &self.graph.edges[edge as usize];
        let (seconds, meters) = (
            f64::from(edge.time_ms) / 1000.0 * part,
            f64::from(edge.length_dm) / 10.0 * part,
        );
        let cost = if self.goal == Goal::Time {
            seconds
        } else {
            meters
        };
        (
            key + cost,
            Leg {
                seconds: leg.seconds + seconds,
                meters: leg.meters + meters,
            },
        )
    }

    /// An edge reached at this cost from the edge `from` (NONE: leaving the start), kept if no cheaper way to it is
    /// known; a drive's search remembers where it came from.
    pub(super) fn reach(&mut self, edge: u32, key: f64, leg: Leg, from: u32) {
        let best = &mut self.best[edge as usize];
        if key < *best {
            if best.is_infinite() {
                self.touched.push(edge);
            }
            *best = key;
            if let Some(pred) = self.pred.get_mut(edge as usize) {
                *pred = from;
            }
            self.heap.push(Entry {
                key,
                leg,
                item: Item::Edge(edge),
            });
        }
    }

    /// A spot reached from the edge `arriving` (NONE: along the start's own stretch) along `edge`: for a table its
    /// index, for a drive the way it came, kept to be traced back.
    pub(super) fn found(&mut self, spot: usize, arriving: u32, edge: u32) -> Item {
        if self.pred.is_empty() {
            return Item::Spot(spot as u32);
        }
        self.vias.push((arriving, edge));
        Item::Via(self.vias.len() as u32 - 1)
    }

    /// A turn's time, from the table.
    pub(super) fn turn(&self, angle: f64) -> f64 {
        self.turns[(angle.round() as i64 + 180).clamp(0, 360) as usize]
    }

    /// The spots looked for on a chain (indexes into `on`).
    pub(super) fn on_chain(&self, chain: u32) -> Range<usize> {
        let start = self.on.partition_point(|(c, _)| *c < chain);
        start..start + self.on[start..].partition_point(|(c, _)| *c == chain)
    }
}
