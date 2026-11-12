//! The drive between spots worked out the slow way, to check the library's search: Bellman-Ford over the same edge
//! states (every state relaxed again until nothing changes), with the same turn rules written out again here.

mod start;

use offline_route_optimizer::geo;
use offline_route_optimizer::graph::Graph;
use offline_route_optimizer::route::{Goal, U_TURN_SECONDS, turn_seconds};
use offline_route_optimizer::snap::Spot;
use start::{cost, start};

/// The least cost (seconds or metres) from each spot to each other.
pub fn table(graph: &Graph, spots: &[Spot], goal: Goal) -> Vec<Vec<Option<f64>>> {
    (0..spots.len())
        .map(|from| one(graph, spots, from, goal))
        .collect()
}

fn one(graph: &Graph, spots: &[Spot], from: usize, goal: Goal) -> Vec<Option<f64>> {
    let (mut dist, mut arrive) = start(graph, spots, from, goal);
    let mut changed = true;
    while changed {
        changed = false;
        for e in 0..graph.edges.len() {
            if dist[e].is_infinite() {
                continue;
            }
            let edge = graph.edges[e];
            let back = |n: usize| {
                graph.edges[n].chain == edge.chain && graph.edges[n].forward != edge.forward
            };
            let open = graph
                .out(edge.head)
                .any(|n| !graph.banned(e as u32, n as u32) && !back(n));
            for n in graph.out(edge.head) {
                if graph.banned(e as u32, n as u32) || (back(n) && open) {
                    continue;
                }
                let next = graph.edges[n];
                let mut wait = 0.0;
                if graph.roads[edge.head as usize] > 2 || back(n) {
                    wait = turn_seconds(geo::turn(
                        f64::from(edge.end_bearing),
                        f64::from(next.start_bearing),
                    ));
                    wait += if back(n) { U_TURN_SECONDS } else { 0.0 };
                }
                let base = dist[e] + if goal == Goal::Time { wait } else { 0.0 };
                if base + cost(graph, n as u32, goal) < dist[n] - 1e-9 {
                    dist[n] = base + cost(graph, n as u32, goal);
                    changed = true;
                }
                for (t, spot) in spots
                    .iter()
                    .enumerate()
                    .filter(|(t, spot)| *t != from && spot.chain == next.chain)
                {
                    let part = if next.forward {
                        spot.share
                    } else {
                        1.0 - spot.share
                    };
                    arrive[t] = arrive[t].min(base + part * cost(graph, n as u32, goal));
                }
            }
        }
    }
    arrive
        .into_iter()
        .map(|cost| cost.is_finite().then_some(cost))
        .collect()
}
