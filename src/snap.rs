//! A place on the road it is reached from: the nearest point of a road a car may use, never a motorway or its ramps,
//! and when the address names its street, the nearest point of that street if one lies within 300 m (an address is
//! reached from its own street even where another road passes closer, as at a corner). Roads are looked for up to
//! 2 km away.

mod closest;

use crate::graph::{Graph, Grid};
use crate::osm::Point;
use crate::roads::is_highway;
use crate::{Error, Result, geo, street};
use closest::closest;

/// Where a place meets the road network.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub chain: u32,
    /// How far along the chain's shape, as a share of its length (0 at its first point, 1 at its last).
    pub share: f64,
    /// The point on the road.
    pub at: Point,
    /// Metres from the place to the road.
    pub off: f64,
}

/// How near the street an address names must be to be taken over a nearer road.
pub const STREET_REACH: f64 = 300.0;
/// How far from a place a road is looked for.
pub const REACH: f64 = 2_000.0;

/// The spot for a place at `at`, with its address if it has one.
pub fn snap(graph: &Graph, at: Point, address: Option<&str>) -> Result<Spot> {
    let want = address
        .and_then(street::street)
        .map(|name| street::key(&name));
    let lat_m = geo::RADIUS.to_radians() * f64::from(graph.grid.cell) / 1e7;
    let cell_m = lat_m * at.lat_degrees().to_radians().cos().min(1.0);
    let mut ring = (STREET_REACH / cell_m).ceil().max(1.0) as i32;
    loop {
        let covered = f64::from(ring) * cell_m;
        let (nearest, named) = search(graph, at, ring, want.as_deref());
        if let Some(spot) = named.filter(|spot| spot.off <= STREET_REACH) {
            return Ok(spot);
        }
        match nearest {
            Some(spot) if spot.off <= covered.min(REACH) => return Ok(spot),
            _ if covered >= REACH => return Err(Error::NoRoad),
            _ => ring *= 2,
        }
    }
}

/// The nearest spot on any road and on the named street, over the cells within `ring` of the place's cell.
fn search(graph: &Graph, at: Point, ring: i32, want: Option<&str>) -> (Option<Spot>, Option<Spot>) {
    let (lat, lon) = (Grid::cell(at.lat), Grid::cell(at.lon));
    let mut chains: Vec<u32> = Vec::new();
    for y in lat - ring..=lat + ring {
        for x in lon - ring..=lon + ring {
            chains.extend_from_slice(graph.grid.chains(y, x));
        }
    }
    chains.sort_unstable();
    chains.dedup();
    let (mut nearest, mut named): (Option<Spot>, Option<Spot>) = (None, None);
    for chain in chains {
        if is_highway(graph.chains[chain as usize].class) {
            continue;
        }
        let spot = closest(graph, chain, at);
        if nearest.is_none_or(|best| spot.off < best.off) {
            nearest = Some(spot);
        }
        let same = want.is_some_and(|want| {
            graph
                .name(chain)
                .is_some_and(|name| street::key(name) == want)
        });
        if same && named.is_none_or(|best| spot.off < best.off) {
            named = Some(spot);
        }
    }
    (nearest, named)
}
