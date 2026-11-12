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

    /// An edge reached at this cost, kept if no cheaper way to it is known.
    pub(super) fn reach(&mut self, edge: u32, key: f64, leg: Leg) {
        let best = &mut self.best[edge as usize];
        if key < *best {
            if best.is_infinite() {
                self.touched.push(edge);
            }
            *best = key;
            self.heap.push(Entry {
                key,
                leg,
                item: Item::Edge(edge),
            });
        }
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
