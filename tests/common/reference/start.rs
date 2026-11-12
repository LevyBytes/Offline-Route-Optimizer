//! Where the slow search begins: the start spot's own stretch of road driven each way, and the spots further along
//! that stretch.

use offline_route_optimizer::graph::{Graph, NONE};
use offline_route_optimizer::route::Goal;
use offline_route_optimizer::snap::Spot;

pub fn cost(graph: &Graph, edge: u32, goal: Goal) -> f64 {
    let edge = &graph.edges[edge as usize];
    if goal == Goal::Time {
        f64::from(edge.time_ms) / 1000.0
    } else {
        f64::from(edge.length_dm) / 10.0
    }
}

/// Each edge's cost and each spot's before any edge is relaxed.
pub fn start(graph: &Graph, spots: &[Spot], from: usize, goal: Goal) -> (Vec<f64>, Vec<f64>) {
    let (mut dist, mut arrive) = (
        vec![f64::INFINITY; graph.edges.len()],
        vec![f64::INFINITY; spots.len()],
    );
    arrive[from] = 0.0;
    let s = spots[from];
    let chain = graph.chains[s.chain as usize];
    for (edge, part) in [(chain.forward, 1.0 - s.share), (chain.backward, s.share)] {
        if edge != NONE {
            dist[edge as usize] = part * cost(graph, edge, goal);
        }
    }
    for (t, spot) in spots
        .iter()
        .enumerate()
        .filter(|(t, spot)| *t != from && spot.chain == s.chain)
    {
        for (edge, ahead) in [
            (chain.forward, spot.share - s.share),
            (chain.backward, s.share - spot.share),
        ] {
            if edge != NONE && ahead >= 0.0 {
                arrive[t] = arrive[t].min(ahead * cost(graph, edge, goal));
            }
        }
    }
    (dist, arrive)
}
