//! A drive's search: the table's (search.rs, expand.rs) from one spot with the other the only one looked for, kept
//! going until that spot's entry leaves the heap, each edge remembering the edge it was reached from.

use super::entry::Item;
use super::search::Search;
use super::{Drive, Goal, trace};
use crate::graph::{Graph, NONE};
use crate::snap::Spot;

impl<'a> Search<'a> {
    /// A search that keeps the edge each edge was reached from, to trace a drive back.
    pub(super) fn tracked(graph: &'a Graph, goal: Goal) -> Self {
        let mut search = Search::new(graph, goal);
        search.pred = vec![NONE; graph.edges.len()];
        search
    }

    /// The drive from one spot to the other, or None past the search's limit; the arrays left ready for the next.
    /// Only an edge reached in this search is traced back through, so `pred` needs no reset.
    pub(super) fn to(&mut self, from: Spot, to: Spot) -> Option<Drive> {
        let spots = [from, to];
        self.on = vec![(to.chain, 1)];
        self.start(&spots, 0);
        let (limit, mut found) = (self.limit(), None);
        while let Some(entry) = self.heap.pop() {
            if entry.key > limit {
                break;
            }
            match entry.item {
                Item::Via(via) => {
                    found = Some((entry.leg, self.vias[via as usize]));
                    break;
                }
                Item::Edge(edge) if entry.key <= self.best[edge as usize] => {
                    self.relax(edge, entry, &spots)
                }
                _ => {}
            }
        }
        let drive = found.map(|(leg, way)| Drive {
            leg,
            points: trace::trace(self.graph, &self.pred, (from, to), way),
        });
        for edge in self.touched.drain(..) {
            self.best[edge as usize] = f64::INFINITY;
        }
        self.heap.clear();
        self.vias.clear();
        drive
    }
}
