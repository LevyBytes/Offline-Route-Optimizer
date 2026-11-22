//! One drive with the road it takes, for a map of the day: the table's own search from one spot until it reaches the
//! other (track.rs), keeping the edge each edge was reached from (4 bytes an edge beside the search's 8), then traced
//! back from the spot to the start (trace.rs). Its leg is the one the table finds for the two spots.

use super::search::Search;
use super::{Goal, Leg, MOST_SPOTS, THREADS, fits};
use crate::graph::Graph;
use crate::osm::Point;
use crate::snap::Spot;
use crate::{Error, Result};

/// One drive and its road: the points from the first spot to the second in driving order (each spot's own point at
/// its end, part of the spots' own stretches, the shapes of the roads between), never one point twice in a row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drive {
    pub leg: Leg,
    pub points: Vec<Point>,
}

/// The drive from `from` to `to` with its road, the leg `table` finds for them; None where it finds none.
/// Refused: a spot `snap` did not make on this graph.
pub fn drive(graph: &Graph, from: &Spot, to: &Spot, goal: Goal) -> Result<Option<Drive>> {
    if !fits(graph, from) || !fits(graph, to) {
        return Err(Error::Input("a spot from another graph"));
    }
    Ok(Search::tracked(graph, goal).to(*from, *to))
}

/// The drive from each spot to the next (the first to the second, the second to the third, on to the last), each
/// with its road; as many searches side by side as a table's, each with its own arrays (12 bytes an edge).
/// Refused: more than MOST_SPOTS spots, and a spot `snap` did not make on this graph.
pub fn drives(graph: &Graph, spots: &[Spot], goal: Goal) -> Result<Vec<Option<Drive>>> {
    if spots.len() > MOST_SPOTS {
        return Err(Error::Limit("drives through more than 1,000 spots"));
    }
    if !spots.iter().all(|spot| fits(graph, spot)) {
        return Err(Error::Input("a spot from another graph"));
    }
    let legs = spots.len().saturating_sub(1);
    if legs == 0 {
        return Ok(Vec::new());
    }
    let threads = THREADS.min(legs);
    let mut found = vec![None; legs];
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|first| {
                scope.spawn(move || {
                    let mut search = Search::tracked(graph, goal);
                    (first..legs)
                        .step_by(threads)
                        .map(|k| (k, search.to(spots[k], spots[k + 1])))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            let done = handle
                .join()
                .map_err(|_| Error::Input("a graph the search cannot use"))?;
            for (k, drive) in done {
                found[k] = drive;
            }
        }
        Ok(found)
    })
}
