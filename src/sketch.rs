//! A sketch of an area's major roads, for a map of a day's places: the stretches of road of the classes asked for that
//! cross a box, found through the grid rather than by reading every road, ramps and other links left out; the
//! stretches of one road (its class and name) joined where they meet end to end, each line cut to the box (it runs on
//! to its first point past an edge) and given fewer points (geo::simplify).

mod bounds;
mod join;

pub use bounds::Bounds;

use crate::graph::{Graph, Grid};
use crate::osm::Point;
use crate::{Error, Result, geo};

/// One line of a sketch: its road's class (0 motorway to 13 service, as roads.rs numbers them) and its points.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub class: u8,
    pub points: Vec<Point>,
}

/// The roads of class `most` and lower crossing `bounds`, links left out, each line with fewer points (none left out
/// more than `tolerance` metres from it), motorways first. Refused: a box turned inside out, or off the earth.
pub fn roads(graph: &Graph, bounds: Bounds, most: u8, tolerance: f64) -> Result<Vec<Line>> {
    if !bounds.usable() {
        return Err(Error::Input("a box turned inside out or off the earth"));
    }
    let chain = |id: &u32| &graph.chains[*id as usize];
    let mut chains = crossing(graph, bounds, |id| {
        let class = graph.chains[id as usize].class;
        class <= most && !link(class)
    });
    chains.sort_unstable_by_key(|id| (chain(id).class, chain(id).name, *id));
    chains.dedup();
    let road = |id: &u32| (chain(id).class, chain(id).name);
    let mut lines = Vec::new();
    for same in chains.chunk_by(|a, b| road(a) == road(b)) {
        let class = chain(&same[0]).class;
        for joined in join::join(graph, same) {
            for run in join::cut(&joined, bounds) {
                let points = geo::simplify(&run, tolerance);
                lines.push(Line { class, points });
            }
        }
    }
    Ok(lines)
}

/// The ramps and other links: motorway_link to tertiary_link (roads.rs).
fn link(class: u8) -> bool {
    class <= 9 && class % 2 == 1
}

/// The chains `wanted` the grid lists in the cells the box covers, some more than once: in each row of cells, the
/// run of its cells from the west edge's to the east edge's (two runs where the box crosses the prime meridian, as
/// cells key their longitude as an unsigned number).
fn crossing(graph: &Graph, bounds: Bounds, wanted: impl Fn(u32) -> bool) -> Vec<u32> {
    let grid = &graph.grid;
    let (west, east) = (Grid::cell(bounds.west), Grid::cell(bounds.east));
    let mut chains = Vec::new();
    let mut take = |row: i32, from: i32, to: i32| {
        let first = grid.keys.partition_point(|key| *key < Grid::key(row, from));
        let last = grid.keys.partition_point(|key| *key <= Grid::key(row, to));
        for k in first..last.max(first) {
            let cell = &grid.pieces[grid.starts[k] as usize..grid.starts[k + 1] as usize];
            chains.extend(cell.iter().copied().filter(|id| wanted(*id)));
        }
    };
    for row in Grid::cell(bounds.south)..=Grid::cell(bounds.north) {
        if west < 0 && east >= 0 {
            take(row, west, -1);
            take(row, 0, east);
        } else {
            take(row, west, east);
        }
    }
    chains
}
