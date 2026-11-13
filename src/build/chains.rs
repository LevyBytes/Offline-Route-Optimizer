//! Joining the roads into a network: a point that ends a road or is shared by two is a junction, the stretch of a
//! road between junctions is a chain, and each direction a car may drive a chain is an edge whose time is its length
//! at the road's speed. A point missing from the file splits its road there, and so does a piece longer than 50 km
//! or across the 180th meridian (none in Nevada is longer than 13 km): such a piece is a mistake in the map.

use std::collections::HashMap;

use super::chain::chain;
use super::collect::Collected;
use super::{Progress, Step};
use crate::geo;
use crate::graph::{Edge, Graph, NONE};
use crate::osm::Point;

/// Whether a piece of road is too long to be real.
fn long(a: Point, b: Point) -> bool {
    geo::meters(a, b) > 50_000.0 || (i64::from(a.lon) - i64::from(b.lon)).abs() > 1_800_000_000
}

pub struct Joined {
    pub graph: Graph,
    /// Each road's chains, by way id.
    pub way_chains: HashMap<i64, Vec<u32>>,
    /// The junction each point became, by its place in Collected.ids (NONE if it is not one).
    pub node_of: Vec<u32>,
}

pub fn join(c: &Collected, progress: &mut Progress<'_>) -> Joined {
    let mut uses = vec![0u8; c.ids.len()];
    for way in &c.ways {
        let last = way.refs.len().saturating_sub(1);
        for (k, id) in way.refs.iter().enumerate() {
            if let Ok(i) = c.ids.binary_search(id) {
                uses[i] = uses[i].saturating_add(if k == 0 || k == last { 2 } else { 1 });
            }
        }
    }
    let mut joined = Joined {
        graph: Graph::default(),
        way_chains: HashMap::new(),
        node_of: vec![NONE; c.ids.len()],
    };
    let (mut names, mut edges) = (HashMap::new(), Vec::new());
    for (index, way) in c.ways.iter().enumerate() {
        if index % 65_536 == 0 {
            progress(Step::Joining, index as u64, c.ways.len() as u64);
        }
        let mut shape: Vec<(usize, Point)> = Vec::new();
        for id in &way.refs {
            let i = c.ids.binary_search(id).expect("every road point is listed");
            let Some(at) = c.positions[i] else {
                chain(&mut joined, &mut names, &mut edges, way, &shape);
                shape.clear();
                continue;
            };
            if shape.last().is_some_and(|(j, _)| *j == i) {
                continue;
            }
            if shape.last().is_some_and(|(_, last)| long(*last, at)) {
                chain(&mut joined, &mut names, &mut edges, way, &shape);
                shape.clear();
            }
            shape.push((i, at));
            if shape.len() > 1 && uses[i] >= 2 {
                chain(&mut joined, &mut names, &mut edges, way, &shape);
                shape = vec![(i, at)];
            }
        }
        chain(&mut joined, &mut names, &mut edges, way, &shape);
    }
    edges.sort_by_key(|edge: &Edge| (edge.tail, edge.chain, !edge.forward));
    let graph = &mut joined.graph;
    graph.first_edge = vec![0; graph.nodes.len() + 1];
    for (index, edge) in edges.iter().enumerate() {
        graph.first_edge[edge.tail as usize + 1] += 1;
        let chain = &mut graph.chains[edge.chain as usize];
        *(if edge.forward {
            &mut chain.forward
        } else {
            &mut chain.backward
        }) = index as u32;
    }
    for node in 0..graph.nodes.len() {
        graph.first_edge[node + 1] += graph.first_edge[node];
    }
    graph.edges = edges;
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort_by_key(|(_, index)| *index);
    graph.names = names.into_iter().map(|(name, _)| name).collect();
    joined
}
