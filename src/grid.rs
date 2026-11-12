//! The grid snapping searches: the map cut into square cells of degrees, each listing the chains with a piece in it.

/// The chains in each cell of `cell` ten-millionths of a degree: pieces[starts[k]..starts[k + 1]] for keys[k].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grid {
    pub cell: i32,
    pub keys: Vec<u64>,
    pub starts: Vec<u32>,
    pub pieces: Vec<u32>,
}

impl Grid {
    pub fn cell(value: i32) -> i32 {
        value.div_euclid(crate::build::CELL)
    }

    pub fn key(lat: i32, lon: i32) -> u64 {
        (u64::from(lat as u32) << 32) | u64::from(lon as u32)
    }

    /// The chains with a piece in a cell.
    pub fn chains(&self, lat: i32, lon: i32) -> &[u32] {
        match self.keys.binary_search(&Grid::key(lat, lon)) {
            Ok(k) => &self.pieces[self.starts[k] as usize..self.starts[k + 1] as usize],
            Err(_) => &[],
        }
    }
}
