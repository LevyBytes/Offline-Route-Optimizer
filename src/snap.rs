//! A place on the road it is reached from: the nearest point of a road a car may use, never a motorway or its ramps,
//! and when the address names its street, the nearest point of that street if one lies within 300 m (an address is
//! reached from its own street even where another road passes closer, as at a corner). Roads are looked for up to
//! 2 km away; a place off the earth is refused, and a map without roads has none near anything.

mod closest;
mod search;

use crate::graph::Graph;
use crate::osm::Point;
use crate::{Error, Result, street};
use search::{rings, search};

/// Where a place meets the road network. Only `snap` makes one, and it remembers the chain's shape it was made on,
/// so a route on another graph (as after the day's new map) is refused rather than taken along the wrong road.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub chain: u32,
    /// How far along the chain's shape, as a share of its length (0 at its first point, 1 at its last).
    pub share: f64,
    /// The point on the road.
    pub at: Point,
    /// Metres from the place to the road.
    pub off: f64,
    /// The chain's first point and its count of points.
    pub(crate) origin: (Point, u32),
}

/// How near the street an address names must be to be taken over a nearer road.
pub const STREET_REACH: f64 = 300.0;
/// How far from a place a road is looked for.
pub const REACH: f64 = 2_000.0;

/// The spot for a place at `at`, with its address if it has one.
pub fn snap(graph: &Graph, at: Point, address: Option<&str>) -> Result<Spot> {
    if !at.valid() {
        return Err(Error::Input("a place off the earth's coordinates"));
    }
    if graph.grid.keys.is_empty() {
        return Err(Error::NoRoad);
    }
    let want = address
        .and_then(street::street)
        .map(|name| street::key(&name));
    let mut reach = STREET_REACH;
    loop {
        let (nearest, named) = search(graph, at, rings(at, reach), want.as_deref());
        if let Some(spot) = named.filter(|spot| spot.off <= STREET_REACH) {
            return Ok(spot);
        }
        match nearest {
            Some(spot) if spot.off <= reach => return Ok(spot),
            _ if reach >= REACH => return Err(Error::NoRoad),
            _ => reach = (reach * 2.0).min(REACH),
        }
    }
}
