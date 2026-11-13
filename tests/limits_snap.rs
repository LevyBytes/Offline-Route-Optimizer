//! Snapping and the route table answer what they cannot use instead of hanging or panicking: a place off the earth,
//! a place at a pole (its rings bounded), a graph without roads, a spot from another graph, more spots than a table
//! takes.

mod common;

use std::sync::mpsc;
use std::time::Duration;

use common::refused::{input, limit};
use common::{Map, temp};
use offline_route_optimizer::graph::Graph;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Goal};
use offline_route_optimizer::{Error, build, snap};

fn graph(map: &Map, name: &str) -> Graph {
    let path = temp(name);
    map.write(&path);
    build::build(&path, "test", &mut |_, _, _| {}).unwrap()
}

/// `work` run on its own thread, which must answer within ten seconds.
fn within<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (send, answer) = mpsc::channel();
    std::thread::spawn(move || send.send(work()));
    answer
        .recv_timeout(Duration::from_secs(10))
        .expect("an answer within ten seconds")
}

#[test]
fn snapping_refuses_places_it_cannot_use() {
    let g = graph(
        &Map::grid(3, &[("highway", "residential")]),
        "snap-limits.osm.pbf",
    );
    let swapped = Point {
        lat: -1_152_000_000,
        lon: 361_000_000,
    };
    assert!(input(snap::snap(&g, swapped, None)));
    let pole = g.clone();
    let pole = within(move || snap::snap(&pole, Point::degrees(90.0, 0.0), None).err());
    assert!(matches!(pole, Some(Error::NoRoad)));
    let empty = within(|| snap::snap(&Graph::default(), Point::degrees(36.1, -115.2), None).err());
    assert!(matches!(empty, Some(Error::NoRoad)));
}

#[test]
fn a_table_refuses_a_spot_from_another_graph_and_too_many_spots() {
    let map = Map::grid(3, &[("highway", "residential")]);
    let mut moved = Map::grid(3, &[("highway", "residential")]);
    moved
        .nodes
        .iter_mut()
        .for_each(|node| node.at.lat += 100_000);
    let (here, there) = (
        graph(&map, "table-here.osm.pbf"),
        graph(&moved, "table-there.osm.pbf"),
    );
    let spot = snap::snap(&here, Point::degrees(36.1005, -115.2), None).unwrap();
    assert_eq!(
        route::table(&here, &[spot, spot], Goal::Time)
            .unwrap()
            .len(),
        2
    );
    assert!(input(route::table(&there, &[spot], Goal::Time)));
    let many = vec![spot; route::MOST_SPOTS + 1];
    assert!(limit(route::table(&here, &many, Goal::Time)));
}
