//! Each part of the graph as a list of fixed-size records (codec.rs), in the file's order.

use std::io::{Read, Write};

use super::bytes::{Put, Take};
use super::codec::{
    chain, edge, point, put_chain, put_edge, put_point, put_u32, u32_at, u32_record,
};
use crate::Result;
use crate::graph::{Graph, Grid};

pub fn put<W: Write>(put: &mut Put<W>, g: &Graph) -> Result<()> {
    put.list(&g.nodes, put_point)?;
    put.list(&g.first_edge, put_u32)?;
    put.list(&g.edges, put_edge)?;
    put.list(&g.chains, put_chain)?;
    put.list(&g.points, put_point)?;
    put.list(&g.roads, |roads, out| out.push(*roads))?;
    put.list(&g.bans, |(from, to), out| {
        [from, to].iter().for_each(|value| put_u32(value, out))
    })?;
    put.raw(&(g.names.len() as u64).to_le_bytes())?;
    for name in &g.names {
        put.text(name)?;
    }
    put.raw(&g.grid.cell.to_le_bytes())?;
    put.list(&g.grid.keys, |key, out| out.extend(key.to_le_bytes()))?;
    put.list(&g.grid.starts, put_u32)?;
    put.list(&g.grid.pieces, put_u32)
}

pub fn take<R: Read>(take: &mut Take<R>) -> Result<Graph> {
    let (nodes, first_edge) = (take.list(point)?, take.list(u32_record)?);
    let (edges, chains, points) = (take.list(edge)?, take.list(chain)?, take.list(point)?);
    let roads = take.list(|b: &[u8; 1]| b[0])?;
    let bans = take.list(|b: &[u8; 8]| (u32_at(b, 0), u32_at(b, 4)))?;
    let count = u64::from_le_bytes(take.raw()?);
    let names = (0..count)
        .map(|_| take.text())
        .collect::<Result<Vec<_>>>()?;
    let cell = i32::from_le_bytes(take.raw()?);
    let keys = take.list(|b: &[u8; 8]| u64::from_le_bytes(*b))?;
    let (starts, pieces) = (take.list(u32_record)?, take.list(u32_record)?);
    let grid = Grid {
        cell,
        keys,
        starts,
        pieces,
    };
    Ok(Graph {
        meta: Default::default(),
        nodes,
        first_edge,
        edges,
        chains,
        points,
        names,
        bans,
        roads,
        grid,
    })
}
