//! Turn restrictions as forbidden pairs of edges at their via junction. "no_..." forbids each edge of the from road
//! arriving there followed by each edge of the to road leaving; "only_..." forbids every other edge leaving. A
//! restriction from a road back onto itself (no_u_turn) forbids only turning back along the same chain. One whose
//! roads or junction the graph lacks (a road cars may not use, a point outside the extract) is left out.

use super::chains::Joined;
use super::collect::Collected;
use crate::graph::{Graph, NONE};

pub fn bans(c: &Collected, j: &Joined) -> Vec<(u32, u32)> {
    let graph = &j.graph;
    let mut bans = Vec::new();
    for r in &c.restrictions {
        let Ok(point) = c.ids.binary_search(&r.via) else {
            continue;
        };
        let via = j.node_of[point];
        if via == NONE {
            continue;
        }
        let arriving: Vec<u32> = edges(graph, j, r.from)
            .filter(|e| graph.edges[*e as usize].head == via)
            .collect();
        let leaving: Vec<u32> = edges(graph, j, r.to)
            .filter(|e| graph.edges[*e as usize].tail == via)
            .collect();
        if arriving.is_empty() || leaving.is_empty() {
            continue;
        }
        for &a in &arriving {
            let chain = graph.edges[a as usize].chain;
            for out in graph.out(via) {
                let out = out as u32;
                let to_road = leaving.contains(&out);
                let back = graph.edges[out as usize].chain == chain && out != a;
                let banned = match (r.only, r.from == r.to) {
                    (false, true) => back,
                    (false, false) => to_road,
                    (true, _) => !to_road,
                };
                if banned {
                    bans.push((a, out));
                }
            }
        }
    }
    bans.sort_unstable();
    bans.dedup();
    bans
}

/// The edges of a road's chains, both ways.
fn edges<'a>(graph: &'a Graph, j: &'a Joined, way: i64) -> impl Iterator<Item = u32> + 'a {
    let chains = j.way_chains.get(&way).map_or(&[][..], Vec::as_slice);
    chains
        .iter()
        .flat_map(move |chain| {
            let chain = &graph.chains[*chain as usize];
            [chain.forward, chain.backward]
        })
        .filter(|edge| *edge != NONE)
}
