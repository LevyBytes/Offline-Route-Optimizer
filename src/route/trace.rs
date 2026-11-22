//! A drive's road, traced back: from the spot's entry to the edge it was reached along, then edge by edge to the one
//! that left the start, and each one's points in driving order: the start's own stretch from its spot on, the
//! stretches between whole, the last one up to the other spot. A stretch's points are cut where `snap` measures a
//! spot's share, by the same lengths along its shape.

use crate::geo;
use crate::graph::{Graph, NONE};
use crate::osm::Point;
use crate::snap::Spot;

/// The points from `from` to `to`, the spot reached from the edge `arriving` (NONE: along the start's own stretch)
/// along the edge `last`; `pred` is the edge each edge was reached from (NONE: it left the start).
pub(super) fn trace(
    graph: &Graph,
    pred: &[u32],
    (from, to): (Spot, Spot),
    (arriving, last): (u32, u32),
) -> Vec<Point> {
    let mut points = vec![from.at];
    if arriving == NONE {
        inside(graph, from.chain, from.share, to.share, &mut points);
    } else {
        // Each edge was reached from one settled before it, so the way back ends; `take` bounds it all the same.
        let back = |edge: &u32| Some(pred[*edge as usize]).filter(|before| *before != NONE);
        let mut edges: Vec<u32> = std::iter::successors(Some(arriving), back)
            .take(pred.len())
            .collect();
        edges.reverse();
        let first = graph.edges[edges[0] as usize];
        let head = if first.forward { 1.0 } else { 0.0 };
        inside(graph, from.chain, from.share, head, &mut points);
        push(&mut points, graph.nodes[first.head as usize]);
        for &edge in &edges[1..] {
            whole(graph, edge, &mut points);
        }
        let last = graph.edges[last as usize];
        push(&mut points, graph.nodes[last.tail as usize]);
        let tail = if last.forward { 0.0 } else { 1.0 };
        inside(graph, to.chain, tail, to.share, &mut points);
    }
    push(&mut points, to.at);
    points
}

/// A chain's shape points strictly between two shares of its length, in order from `from` to `to`.
fn inside(graph: &Graph, chain: u32, from: f64, to: f64, out: &mut Vec<Point>) {
    let shape = graph.shape(chain);
    let mut along = vec![0.0; shape.len()];
    for k in 1..shape.len() {
        along[k] = along[k - 1] + geo::meters(shape[k - 1], shape[k]);
    }
    let total = along[shape.len() - 1];
    if total <= 0.0 {
        return;
    }
    let (low, high) = (from.min(to), from.max(to));
    let between = |k: &usize| low < along[*k] / total && along[*k] / total < high;
    if from <= to {
        (0..shape.len())
            .filter(between)
            .for_each(|k| push(out, shape[k]));
    } else {
        (0..shape.len())
            .rev()
            .filter(between)
            .for_each(|k| push(out, shape[k]));
    }
}

/// An edge's whole stretch in the direction it is driven.
fn whole(graph: &Graph, edge: u32, out: &mut Vec<Point>) {
    let edge = graph.edges[edge as usize];
    let shape = graph.shape(edge.chain);
    if edge.forward {
        shape.iter().for_each(|point| push(out, *point));
    } else {
        shape.iter().rev().for_each(|point| push(out, *point));
    }
}

/// A point added unless it is the last one again.
fn push(out: &mut Vec<Point>, point: Point) {
    if out.last() != Some(&point) {
        out.push(point);
    }
}
