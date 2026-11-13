//! The build keeps its memory to the map: a piece of road is listed in the grid cells it crosses, not every cell of
//! its box; a piece longer than 50 km, across the 180th meridian or to a point off the earth splits its road; a road
//! made to pass one junction again and again forbids no flood of turns; a posted speed outside 5 to 200 km/h is not
//! taken.

mod common;

use common::{Map, tags, temp};
use offline_route_optimizer::graph::{Graph, Grid};
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::{build, geo, roads};

fn graph(map: &Map, name: &str) -> Graph {
    let path = temp(name);
    map.write(&path);
    build::build(&path, "test", &mut |_, _, _| {}).unwrap()
}

#[test]
fn a_piece_is_listed_in_the_cells_it_crosses() {
    // 0.3 degrees north and east, about 43 km: its box holds 121 by 121 cells, the piece touches two a row.
    let mut map = Map::default();
    map.node(1, 36.0, -115.3);
    map.node(2, 36.3, -115.0);
    map.way(10, &[1, 2], &[("highway", "residential")]);
    let g = graph(&map, "diagonal.osm.pbf");
    let cells = g.grid.keys.len();
    assert!((121..=2 * 121).contains(&cells), "{cells} cells");
    let (a, b) = (Point::degrees(36.0, -115.3), Point::degrees(36.3, -115.0));
    for k in 0..=10_000 {
        let at = geo::between(a, b, f64::from(k) / 10_000.0);
        let listed = g.grid.chains(Grid::cell(at.lat), Grid::cell(at.lon));
        assert_eq!(listed, &[0], "{at:?}");
    }
}

#[test]
fn impossible_pieces_split_their_roads() {
    let mut map = Map::grid(2, &[("highway", "residential")]);
    map.node(1, 0.0, 0.0);
    map.node(2, 150.0, -115.2);
    map.node(3, 36.1, 179.999);
    map.node(4, 36.1, -179.999);
    let road = [("highway", "residential")];
    map.way(90, &[1000, 1, 1001], &road);
    map.way(91, &[1100, 2, 1101], &road);
    map.way(92, &[3, 4], &road);
    let g = graph(&map, "impossible.osm.pbf");
    // The four streets of the grid stay; the three roads keep no piece at all.
    assert_eq!(g.chains.len(), 4);
    assert!(g.grid.keys.len() < 10, "{} cells", g.grid.keys.len());
}

#[test]
fn a_road_through_one_junction_again_and_again_bans_no_flood_of_turns() {
    let mut map = Map::grid(2, &[("highway", "residential")]);
    for k in 0..999 {
        map.node(2000 + k, 36.1 - 0.0001 * (k + 1) as f64, -115.2001);
    }
    let out_and_back = (0..999).flat_map(|k| [1000, 2000 + k]);
    map.way(
        90,
        &out_and_back.chain([1000]).collect::<Vec<_>>(),
        &[("highway", "residential")],
    );
    for id in 0..100 {
        map.restriction(500 + id, 90, 1000, 10, "only_straight_on");
    }
    map.restriction(700, 10, 1001, 51, "no_left_turn");
    let g = graph(&map, "looping.osm.pbf");
    assert!(g.chains.len() > 999);
    assert!(
        !g.bans.is_empty() && g.bans.len() <= 4_096,
        "{} bans",
        g.bans.len()
    );
}

#[test]
fn a_posted_speed_must_be_one_a_road_could_have() {
    let tiny = format!("0.{}3", "0".repeat(323));
    for value in [tiny.as_str(), "0", "999999999", "3 mph", "125 mph", "201"] {
        assert_eq!(roads::maxspeed(value), None, "{value}");
    }
    assert_eq!(roads::maxspeed("5"), Some(5.0));
    let alley = tags(&[
        ("highway", "service"),
        ("service", "alley"),
        ("maxspeed", &tiny),
    ]);
    let road = roads::road(&alley).unwrap();
    assert_eq!((road.forward, road.backward), (7.5, 7.5));
}
