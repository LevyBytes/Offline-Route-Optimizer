//! The road network as routing uses it: junctions (nodes), the stretches of road between them (chains, with their
//! shape and street name) and each way a car may pass along a chain (edges, with time and length), plus the turns
//! the map forbids and a grid of road pieces for finding the road nearest a place.

use std::ops::Range;

pub use crate::grid::Grid;
use crate::osm::Point;

/// No edge, no name.
pub const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub tail: u32,
    pub head: u32,
    pub chain: u32,
    /// Along the chain's shape (true) or against it.
    pub forward: bool,
    pub time_ms: u32,
    pub length_dm: u32,
    /// The direction of travel leaving the tail and arriving at the head, in whole degrees from north.
    pub start_bearing: u16,
    pub end_bearing: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    /// Its shape: points[first..first + count].
    pub first: u32,
    pub count: u32,
    pub name: u32,
    pub class: u8,
    pub length_dm: u32,
    /// Its edge each way (NONE where the road is one-way the other way).
    pub forward: u32,
    pub backward: u32,
}

/// Where the map came from and how current it is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    pub region: String,
    /// Seconds since 1970 (UTC) of the data's last change, from the file's replication state.
    pub data_time: i64,
    pub sequence: i64,
    pub base_url: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub meta: Meta,
    pub nodes: Vec<Point>,
    /// Each node's edges leaving it: edges[first_edge[n]..first_edge[n + 1]].
    pub first_edge: Vec<u32>,
    pub edges: Vec<Edge>,
    pub chains: Vec<Chain>,
    pub points: Vec<Point>,
    pub names: Vec<String>,
    /// Forbidden turns as (arriving edge, leaving edge), sorted.
    pub bans: Vec<(u32, u32)>,
    /// How many chains meet at each node (the turn cost applies where more than two do).
    pub roads: Vec<u8>,
    pub grid: Grid,
}

impl Graph {
    pub fn out(&self, node: u32) -> Range<usize> {
        let node = node as usize;
        self.first_edge[node] as usize..self.first_edge[node + 1] as usize
    }

    pub fn banned(&self, arriving: u32, leaving: u32) -> bool {
        self.bans.binary_search(&(arriving, leaving)).is_ok()
    }

    pub fn shape(&self, chain: u32) -> &[Point] {
        let chain = &self.chains[chain as usize];
        &self.points[chain.first as usize..(chain.first + chain.count) as usize]
    }

    pub fn name(&self, chain: u32) -> Option<&str> {
        let name = self.chains[chain as usize].name;
        (name != NONE).then(|| self.names[name as usize].as_str())
    }
}
