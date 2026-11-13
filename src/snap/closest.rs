//! The point of a road nearest a place: each piece of its shape in turn, the place projected onto it.

use super::Spot;
use crate::geo;
use crate::graph::Graph;
use crate::osm::Point;

/// The point of a chain nearest a place.
pub(super) fn closest(graph: &Graph, chain: u32, at: Point) -> Spot {
    let shape = graph.shape(chain);
    let lengths: Vec<f64> = shape
        .windows(2)
        .map(|pair| geo::meters(pair[0], pair[1]))
        .collect();
    let total: f64 = lengths.iter().sum();
    let origin = (shape[0], shape.len() as u32);
    let (mut best, mut before) = (
        Spot {
            chain,
            share: 0.0,
            at: shape[0],
            off: f64::INFINITY,
            origin,
        },
        0.0,
    );
    for (k, pair) in shape.windows(2).enumerate() {
        let (t, off) = geo::project(at, pair[0], pair[1]);
        if off < best.off {
            let share = if total > 0.0 {
                (before + t * lengths[k]) / total
            } else {
                0.0
            };
            best = Spot {
                chain,
                share,
                at: geo::between(pair[0], pair[1], t),
                off,
                origin,
            };
        }
        before += lengths[k];
    }
    best
}
