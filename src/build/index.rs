//! The grid snapping searches: the map cut into cells of 0.0025 degrees (about 280 m north to south, 225 m east to
//! west in Nevada), each listing the chains with a piece inside it. A piece is listed in every cell it passes through
//! or touches, so a search over the cells around a place finds every road passing near it. The grid holds at most
//! four cells a point and 2^20 more (Nevada's roads take 0.4 a point), so a map made to fill memory is refused.

use super::{Progress, Step};
use crate::graph::{Graph, Grid};
use crate::osm::Point;
use crate::{Error, Result};

/// The cell size in ten-millionths of a degree.
pub const CELL: i32 = 25_000;

pub fn grid(graph: &Graph, progress: &mut Progress<'_>) -> Result<Grid> {
    let (mut pairs, mut cells) = (Vec::new(), Vec::new());
    let most = graph.points.len().saturating_mul(4).saturating_add(1 << 20);
    let total = graph.chains.len() as u64;
    for chain in 0..graph.chains.len() {
        if chain % 65_536 == 0 {
            progress(Step::Indexing, chain as u64, total);
        }
        cells.clear();
        for pair in graph.shape(chain as u32).windows(2) {
            crossed(pair[0], pair[1], &mut cells);
            if pairs.len() + cells.len() > most {
                return Err(Error::Limit("a road grid past four cells a point"));
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
    Ok(grid)
}

/// The cells a straight piece from `a` to `b` passes through or touches: in each row of cells it reaches, those from
/// the longitude where it enters the row to the one where it leaves.
fn crossed(a: Point, b: Point, cells: &mut Vec<u64>) {
    let (a, b) = if a.lat <= b.lat { (a, b) } else { (b, a) };
    let (lat, lon) = (i64::from(a.lat), i64::from(a.lon));
    let (rise, run) = (i64::from(b.lat) - lat, i128::from(i64::from(b.lon) - lon));
    // The longitude at a latitude between the ends, rounded down (which keeps it in the same cell).
    let at = |y: i64| match rise {
        0 => lon,
        _ => lon + (run * i128::from(y - lat)).div_euclid(i128::from(rise)) as i64,
    };
    let (cell, size) = (
        |value: i64| value.div_euclid(i64::from(CELL)) as i32,
        i64::from(CELL),
    );
    for row in Grid::cell(a.lat)..=Grid::cell(b.lat) {
        let low = lat.max(i64::from(row) * size);
        let high = i64::from(b.lat).min((i64::from(row) + 1) * size);
        let (first, last) = match rise {
            0 => (lon.min(i64::from(b.lon)), lon.max(i64::from(b.lon))),
            _ => (at(low).min(at(high)), at(low).max(at(high))),
        };
        cells.extend((cell(first)..=cell(last)).map(|column| Grid::key(row, column)));
    }
}
