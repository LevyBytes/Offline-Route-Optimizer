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

use crate::graph::Graph;
use crate::snap::Spot;

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

/// The leg from each spot to each other (None where no road joins them within the limit; a spot to itself is 0).
pub fn table(graph: &Graph, spots: &[Spot], goal: Goal) -> Vec<Vec<Option<Leg>>> {
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
            for (from, row) in handle.join().expect("a search thread ends") {
                rows[from] = row;
            }
        }
    });
    rows
}

/// OSRM's turn duration in seconds for a turn of `angle` degrees (positive to the right), driving on the right.
pub fn turn_seconds(angle: f64) -> f64 {
    const PENALTY: f64 = 7.5;
    const BIAS: f64 = 1.075;
    let x = if angle >= 0.0 {
        (13.0 / BIAS) * angle / 180.0 - 6.5 * BIAS
    } else {
        (13.0 * BIAS) * -angle / 180.0 - 6.5 / BIAS
    };
    PENALTY / (1.0 + (-x).exp())
}

/// The extra time a U-turn takes on top of its turn.
pub const U_TURN_SECONDS: f64 = 20.0;
