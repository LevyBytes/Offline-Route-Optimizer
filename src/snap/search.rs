//! The cells around a place and the roads they list: the nearest spot on any road, and on the street an address names.

use super::Spot;
use super::closest::closest;
use crate::build::CELL;
use crate::graph::{Graph, Grid};
use crate::osm::Point;
use crate::roads::is_highway;
use crate::{geo, street};

/// How many cells north and south, and east and west, of the place's cell hold everything within `reach` metres.
/// Cells narrow toward the poles, so their width is taken at the edge nearest the pole, and a ring east and west
/// never passes the whole way round (72,000 cells either side).
pub(super) fn rings(at: Point, reach: f64) -> (i32, i32) {
    let degrees = f64::from(CELL) / 1e7;
    let tall = geo::RADIUS * degrees.to_radians();
    let north = (reach / tall).ceil().max(1.0);
    let edge = (at.lat_degrees().abs() + north * degrees).min(90.0);
    let east = (reach / (tall * edge.to_radians().cos())).ceil();
    (north as i32, east.clamp(1.0, 72_000.0) as i32)
}

/// The nearest spot on any road and on the named street, over the cells within the rings around the place's cell.
pub(super) fn search(
    graph: &Graph,
    at: Point,
    rings: (i32, i32),
    want: Option<&str>,
) -> (Option<Spot>, Option<Spot>) {
    let (lat, lon) = (Grid::cell(at.lat), Grid::cell(at.lon));
    let mut chains: Vec<u32> = Vec::new();
    for y in lat - rings.0..=lat + rings.0 {
        for x in lon - rings.1..=lon + rings.1 {
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
