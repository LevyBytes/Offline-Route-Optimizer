//! The grid snapping searches: the map cut into cells of 0.0025 degrees (about 280 m north to south, 225 m east to
//! west in Nevada), each listing the chains with a piece inside it (a piece is listed in every cell its bounding box
//! touches, so a search over the cells around a place finds every road passing near it).

use super::{Progress, Step};
use crate::graph::{Graph, Grid};

/// The cell size in ten-millionths of a degree.
pub const CELL: i32 = 25_000;

pub fn grid(graph: &Graph, progress: &mut Progress<'_>) -> Grid {
    let (mut pairs, mut cells) = (Vec::new(), Vec::new());
    let total = graph.chains.len() as u64;
    for chain in 0..graph.chains.len() {
        if chain % 65_536 == 0 {
            progress(Step::Indexing, chain as u64, total);
        }
        cells.clear();
        for pair in graph.shape(chain as u32).windows(2) {
            let (lat0, lat1) = order(Grid::cell(pair[0].lat), Grid::cell(pair[1].lat));
            let (lon0, lon1) = order(Grid::cell(pair[0].lon), Grid::cell(pair[1].lon));
            for lat in lat0..=lat1 {
                cells.extend((lon0..=lon1).map(|lon| Grid::key(lat, lon)));
            }
        }
        cells.sort_unstable();
        cells.dedup();
        pairs.extend(cells.iter().map(|key| (*key, chain as u32)));
    }
    progress(Step::Indexing, total, total);
    pairs.sort_unstable();
    let mut grid = Grid {
        cell: CELL,
        keys: Vec::new(),
        starts: Vec::new(),
        pieces: Vec::with_capacity(pairs.len()),
    };
    for (key, chain) in pairs {
        if grid.keys.last() != Some(&key) {
            grid.keys.push(key);
            grid.starts.push(grid.pieces.len() as u32);
        }
        grid.pieces.push(chain);
    }
    grid.starts.push(grid.pieces.len() as u32);
    grid
}

fn order(a: i32, b: i32) -> (i32, i32) {
    if a <= b { (a, b) } else { (b, a) }
}
