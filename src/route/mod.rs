//! The drive between spots, searched over edges rather than junctions (Dijkstra 1959 on the edge graph) so each turn
//! costs what it costs and a forbidden turn is never taken. Turns follow OSRM's car profile (profiles/car.lua, read
//! 2026-11-12): where more than two roads meet, up to 7.5 s, a right turn cheaper than a left (its sigmoid with
//! turn_bias 1.075); a U-turn only where the road ends or every other way is forbidden, 20 s more. A spot's own
//! stretch of road is driven in part. One search from each spot reaches all the others and stops there; a spot not
//! reached within 6 hours (or 600 km) has no leg.

mod costs;
mod entry;
mod expand;
mod search;
mod turns;

use crate::graph::Graph;
use crate::snap::Spot;
use crate::{Error, Result};

pub use turns::{U_TURN_SECONDS, turn_seconds};

/// What the search keeps least: the drive's time, or its length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    Time,
    Distance,
}

/// One drive: how long it takes and how far it goes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Leg {
    pub seconds: f64,
    pub meters: f64,
}

/// The farthest a search goes: 6 hours, or 600 km.
pub const LIMIT_SECONDS: f64 = 6.0 * 3600.0;
pub const LIMIT_METERS: f64 = 600_000.0;

/// How many searches run side by side, each with its own arrays (about 8 bytes an edge). A fixed number: asking the
/// system for its core count reads cgroup files Android's SELinux policy denies an app.
pub const THREADS: usize = 4;

/// The most spots one table takes: a search from each and a leg for each pair, so 1,000 spots take 24 MB of legs.
pub const MOST_SPOTS: usize = 1_000;

/// The leg from each spot to each other (None where no road joins them within the limit; a spot to itself is 0).
/// Refused: more than MOST_SPOTS spots, and a spot `snap` did not make on this graph.
pub fn table(graph: &Graph, spots: &[Spot], goal: Goal) -> Result<Vec<Vec<Option<Leg>>>> {
    if spots.len() > MOST_SPOTS {
        return Err(Error::Limit("a table of more than 1,000 spots"));
    }
    if !spots.iter().all(|spot| fits(graph, spot)) {
        return Err(Error::Input("a spot from another graph"));
    }
    let threads = THREADS.min(spots.len()).max(1);
    let mut rows = vec![Vec::new(); spots.len()];
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|first| {
                scope.spawn(move || {
                    let mut search = search::Search::new(graph, goal);
                    (first..spots.len())
                        .step_by(threads)
                        .map(|from| (from, search.from(spots, from)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            let found = handle
                .join()
                .map_err(|_| Error::Input("a graph the search cannot use"))?;
            for (from, row) in found {
                rows[from] = row;
            }
        }
        Ok(rows)
    })
}

/// Whether a spot was made on this graph: its chain is there with the shape it remembers, its share 0 to 1.
fn fits(graph: &Graph, spot: &Spot) -> bool {
    let Some(chain) = graph.chains.get(spot.chain as usize) else {
        return false;
    };
    let shape = graph.points.get(chain.first as usize..);
    let shape = shape.and_then(|rest| rest.get(..chain.count as usize));
    let origin = shape.and_then(|shape| Some((*shape.first()?, shape.len() as u32)));
    (0.0..=1.0).contains(&spot.share) && origin == Some(spot.origin)
}
