//! A drive answers what it cannot use instead of searching on: no drive between roads that never meet, a drive from a
//! spot to itself, a spot from another graph and more spots than drives in a row take refused.

mod common;

use common::Map;
use common::refused::{input, limit};
use common::town::graph;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Goal, Leg};
use offline_route_optimizer::snap;

#[test]
fn roads_that_never_meet_have_no_drive_and_another_graphs_spot_is_refused() {
    let mut map = Map::default();
    map.node(1, 36.1, -115.2);
    map.node(2, 36.1, -115.199);
    map.node(3, 36.11, -115.2);
    map.node(4, 36.11, -115.199);
    map.way(1, &[1, 2], &[("highway", "residential")]);
    map.way(2, &[3, 4], &[("highway", "residential")]);
    let g = graph(&map, "drive-apart.osm.pbf");
    let a = snap::snap(&g, Point::degrees(36.1, -115.1995), None).unwrap();
    let b = snap::snap(&g, Point::degrees(36.11, -115.1995), None).unwrap();
    assert_eq!(route::drive(&g, &a, &b, Goal::Time).unwrap(), None);
    assert_eq!(
        route::drives(&g, &[a, b, a], Goal::Time).unwrap(),
        [None, None]
    );
    let same = route::drive(&g, &a, &a, Goal::Time).unwrap().unwrap();
    assert_eq!((same.leg, same.points), (Leg::default(), vec![a.at]));
    assert_eq!(route::drives(&g, &[a], Goal::Time).unwrap(), []);
    let other = graph(
        &Map::grid(3, &[("highway", "residential")]),
        "drive-other.osm.pbf",
    );
    assert!(input(route::drive(&other, &a, &b, Goal::Time)));
    assert!(input(route::drives(&other, &[a, b], Goal::Time)));
    let many = vec![a; route::MOST_SPOTS + 1];
    assert!(limit(route::drives(&g, &many, Goal::Time)));
}
