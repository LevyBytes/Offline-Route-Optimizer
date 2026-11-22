//! A road's stretches joined end to end into lines, and a line cut to a box.

use std::collections::{HashMap, VecDeque};

use super::Bounds;
use crate::graph::Graph;
use crate::osm::Point;

/// The chains of one road (one class and name) as lines: each chain's shape, joined at either end to another's that
/// meets it there for as long as one does (at a fork, the first found; the others begin lines of their own).
pub(super) fn join(graph: &Graph, chains: &[u32]) -> Vec<Vec<Point>> {
    let mut ends: HashMap<Point, Vec<usize>> = HashMap::new();
    for (k, chain) in chains.iter().enumerate() {
        let shape = graph.shape(*chain);
        ends.entry(shape[0]).or_default().push(k);
        ends.entry(shape[shape.len() - 1]).or_default().push(k);
    }
    let mut used = vec![false; chains.len()];
    let mut lines = Vec::new();
    for k in 0..chains.len() {
        if used[k] {
            continue;
        }
        used[k] = true;
        let mut line: VecDeque<Point> = graph.shape(chains[k]).iter().copied().collect();
        for back in [true, false] {
            grow(graph, chains, (&ends, &mut used), &mut line, back);
        }
        lines.push(line.into());
    }
    lines
}

/// Chains added at one end of a line (its back, or its front) while another meets it there.
fn grow(
    graph: &Graph,
    chains: &[u32],
    (ends, used): (&HashMap<Point, Vec<usize>>, &mut [bool]),
    line: &mut VecDeque<Point>,
    back: bool,
) {
    loop {
        let end = if back { line.back() } else { line.front() };
        let Some(&end) = end else { return };
        let next = ends
            .get(&end)
            .and_then(|list| list.iter().find(|k| !used[**k]));
        let Some(&k) = next else { return };
        used[k] = true;
        let shape = graph.shape(chains[k]);
        // The chain's points from the end that meets the line on, without that point again.
        let points: Vec<Point> = if shape[0] == end {
            shape[1..].to_vec()
        } else {
            shape[..shape.len() - 1].iter().rev().copied().collect()
        };
        for point in points {
            if back {
                line.push_back(point);
            } else {
                line.push_front(point);
            }
        }
    }
}

/// The runs of a line's pieces that cross the box (a piece crossing when its own box overlaps it), each from the
/// first piece in to the last, so a line runs on to its first point past an edge.
pub(super) fn cut(points: &[Point], bounds: Bounds) -> Vec<Vec<Point>> {
    let mut runs = Vec::new();
    let mut run: Vec<Point> = Vec::new();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let rows = a.lat.min(b.lat) <= bounds.north && a.lat.max(b.lat) >= bounds.south;
        let columns = a.lon.min(b.lon) <= bounds.east && a.lon.max(b.lon) >= bounds.west;
        if rows && columns {
            if run.is_empty() {
                run.push(a);
            }
            run.push(b);
        } else if !run.is_empty() {
            runs.push(std::mem::take(&mut run));
        }
    }
    if !run.is_empty() {
        runs.push(run);
    }
    runs
}
