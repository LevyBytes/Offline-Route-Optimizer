//! A graph read from a file is used only once every index in it points inside its list, so a damaged or mismatched
//! file is refused instead of read past an end.

use crate::graph::{Chain, Graph, NONE};
use crate::{Error, Result};

fn sorted<T: PartialOrd>(list: &[T]) -> bool {
    list.windows(2).all(|pair| pair[0] <= pair[1])
}

fn ranges(starts: &[u32], groups: usize, items: usize) -> bool {
    starts.len() == groups + 1
        && starts[0] == 0
        && sorted(starts)
        && starts[groups] as usize == items
}

fn chain_ok(g: &Graph, index: usize, c: &Chain) -> bool {
    let shape = c
        .first
        .checked_add(c.count)
        .is_some_and(|end| end as usize <= g.points.len());
    let mine = |edge: u32, forward: bool| {
        let e = g.edges.get(edge as usize);
        edge == NONE || e.is_some_and(|e| e.chain == index as u32 && e.forward == forward)
    };
    let named = c.name == NONE || (c.name as usize) < g.names.len();
    let driven = c.forward != NONE || c.backward != NONE;
    shape && c.count >= 2 && named && driven && mine(c.forward, true) && mine(c.backward, false)
}

pub fn check(g: &Graph) -> Result<()> {
    let bad = |what| Err(Error::Format(what));
    let (nodes, edges, chains) = (
        g.nodes.len() as u32,
        g.edges.len() as u32,
        g.chains.len() as u32,
    );
    if !ranges(&g.first_edge, g.nodes.len(), g.edges.len()) || g.roads.len() != g.nodes.len() {
        return bad("graph file nodes");
    }
    for (index, e) in g.edges.iter().enumerate() {
        let tail = e.tail as usize;
        let leaving = e.tail < nodes
            && (g.first_edge[tail]..g.first_edge[tail + 1]).contains(&(index as u32));
        if !leaving || e.head >= nodes || e.chain >= chains {
            return bad("graph file edges");
        }
    }
    if !g
        .chains
        .iter()
        .enumerate()
        .all(|(index, c)| chain_ok(g, index, c))
    {
        return bad("graph file chains");
    }
    if !sorted(&g.bans)
        || g.bans
            .iter()
            .any(|(from, to)| *from >= edges || *to >= edges)
    {
        return bad("graph file turns");
    }
    let grid = &g.grid;
    let keys = grid.keys.windows(2).all(|pair| pair[0] < pair[1]);
    let starts = ranges(&grid.starts, grid.keys.len(), grid.pieces.len());
    if grid.cell != crate::build::CELL
        || !keys
        || !starts
        || grid.pieces.iter().any(|chain| *chain >= chains)
    {
        return bad("graph file grid");
    }
    Ok(())
}
