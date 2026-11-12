//! Building the road graph from an OpenStreetMap extract: the drivable roads and their points (two reads of the
//! file), the network of junctions, chains and edges, the turn restrictions as forbidden pairs of edges, and the
//! grid snapping searches. A file of 100 MB (Nevada) builds in about a minute on a phone; memory grows with the
//! number of road points (about 40 bytes each while building).

mod bans;
mod chain;
mod chains;
mod collect;
mod index;
mod points;

use std::path::Path;

use crate::Result;
use crate::graph::{Graph, Meta, NONE};

pub use index::CELL;

/// The stage a build is in, reported with how far it is (done of total).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Reading the roads and restrictions: bytes of the file read.
    Roads,
    /// Reading the points the roads use: node blocks read.
    Points,
    /// Joining the roads into a network: roads joined.
    Joining,
    /// Listing the roads in the grid: chains listed.
    Indexing,
}

/// Where a build reports its progress: the step, then done and total.
pub type Progress<'a> = dyn FnMut(Step, u64, u64) + 'a;

/// The road graph of an extract, for `region` (a name the caller keeps, e.g. "nevada").
pub fn build(path: &Path, region: &str, progress: &mut Progress<'_>) -> Result<Graph> {
    let collected = collect::collect(path, progress)?;
    let mut joined = chains::join(&collected, progress);
    joined.graph.bans = bans::bans(&collected, &joined);
    let header = &collected.header;
    joined.graph.meta = Meta {
        region: region.to_owned(),
        data_time: header.replication_time.unwrap_or(0),
        sequence: header.replication_sequence.unwrap_or(0),
        base_url: header.replication_url.clone().unwrap_or_default(),
    };
    drop(collected);
    let mut graph = joined.graph;
    graph.roads = roads(&graph);
    graph.grid = index::grid(&graph, progress);
    Ok(graph)
}

/// How many chains meet at each junction (a chain that starts and ends there counts twice).
fn roads(graph: &Graph) -> Vec<u8> {
    let mut roads = vec![0u8; graph.nodes.len()];
    for chain in &graph.chains {
        let edge = if chain.forward != NONE {
            chain.forward
        } else {
            chain.backward
        };
        let edge = &graph.edges[edge as usize];
        for node in [edge.tail, edge.head] {
            roads[node as usize] = roads[node as usize].saturating_add(1);
        }
    }
    roads
}
