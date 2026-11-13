//! One chain: the stretch of a road between two junctions (or the ends of what the file holds of it), its shape and
//! length, its street name (each name kept once), and an edge each way a car may drive it, timed at the road's speed
//! in that direction, with the direction it leaves and arrives in for the turns at either end.

use std::collections::HashMap;

use super::chains::Joined;
use super::collect::RoadWay;
use crate::geo;
use crate::graph::{Chain, Edge, NONE};
use crate::osm::Point;

pub fn chain(
    j: &mut Joined,
    names: &mut HashMap<String, u32>,
    edges: &mut Vec<Edge>,
    way: &RoadWay,
    shape: &[(usize, Point)],
) {
    // A chain needs two points and a direction a car may drive (so every chain has an edge).
    if shape.len() < 2 || (way.road.forward <= 0.0 && way.road.backward <= 0.0) {
        return;
    }
    let mut node = |i: usize, at: Point| {
        if j.node_of[i] == NONE {
            j.node_of[i] = j.graph.nodes.len() as u32;
            j.graph.nodes.push(at);
        }
        j.node_of[i]
    };
    let (first, last) = (shape[0], shape[shape.len() - 1]);
    let (a, b) = (node(first.0, first.1), node(last.0, last.1));
    let points: Vec<Point> = shape.iter().map(|(_, at)| *at).collect();
    let metres: f64 = points
        .windows(2)
        .map(|pair| geo::meters(pair[0], pair[1]))
        .sum();
    let id = j.graph.chains.len() as u32;
    let next = names.len() as u32;
    let name = way
        .name
        .as_ref()
        .map_or(NONE, |name| *names.entry(name.clone()).or_insert(next));
    let length_dm = (metres * 10.0).round() as u32;
    let first_point = j.graph.points.len() as u32;
    j.graph.points.extend_from_slice(&points);
    j.graph.chains.push(Chain {
        first: first_point,
        count: points.len() as u32,
        name,
        class: way.road.class,
        length_dm,
        forward: NONE,
        backward: NONE,
    });
    let n = points.len();
    let degrees = |from: Point, to: Point| geo::bearing(from, to).round() as u16 % 360;
    let time = |speed: f64| (metres * 3600.0 / speed).round() as u32;
    if way.road.forward > 0.0 {
        edges.push(Edge {
            tail: a,
            head: b,
            chain: id,
            forward: true,
            time_ms: time(way.road.forward),
            length_dm,
            start_bearing: degrees(points[0], points[1]),
            end_bearing: degrees(points[n - 2], points[n - 1]),
        });
    }
    if way.road.backward > 0.0 {
        edges.push(Edge {
            tail: b,
            head: a,
            chain: id,
            forward: false,
            time_ms: time(way.road.backward),
            length_dm,
            start_bearing: degrees(points[n - 1], points[n - 2]),
            end_bearing: degrees(points[1], points[0]),
        });
    }
    j.way_chains.entry(way.id).or_default().push(id);
}
