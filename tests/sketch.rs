//! A sketch of an area's major roads: only the classes asked for and never a link, each road's stretches joined into
//! one line with fewer points and cut to the box; a box turned inside out refused; a box around places grown by its
//! margin, and kept on the earth.

mod common;

use common::Map;
use common::town::graph;
use offline_route_optimizer::geo;
use offline_route_optimizer::graph::Graph;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::sketch::{self, Bounds, Line};

/// A 6 by 6 town: row 0 a primary road, column 0 a motorway ramp, column 1 a secondary road, the rest residential.
fn town() -> Graph {
    let mut map = Map::grid(6, &[]);
    for (k, way) in map.ways.iter_mut().enumerate() {
        let class = match k {
            0 => "primary",
            6 => "motorway_link",
            7 => "secondary",
            _ => "residential",
        };
        way.tags.push(("highway".into(), class.into()));
    }
    graph(&map, "sketch.osm.pbf")
}

/// A line's two ends, the west or south one first.
fn ends(line: &Line) -> (Point, Point) {
    let (a, b) = (line.points[0], line.points[line.points.len() - 1]);
    if (a.lon, a.lat) <= (b.lon, b.lat) {
        (a, b)
    } else {
        (b, a)
    }
}

#[test]
fn only_the_classes_asked_for_joined_cut_to_the_box_and_simplified() {
    let g = town();
    let all = Bounds::around(&[Point::degrees(36.1025, -115.1975)], 1_000.0).unwrap();
    let primary = sketch::roads(&g, all, 4, 1.0).unwrap();
    assert_eq!(primary.len(), 1, "{primary:?}");
    assert_eq!(primary[0].class, 4);
    let row = (Point::degrees(36.1, -115.2), Point::degrees(36.1, -115.195));
    assert_eq!((primary[0].points.len(), ends(&primary[0])), (2, row));
    let more = sketch::roads(&g, all, 6, 1.0).unwrap();
    let classes: Vec<u8> = more.iter().map(|line| line.class).collect();
    assert_eq!(classes, [4, 6], "the ramp is left out");
    let column = (
        Point::degrees(36.1, -115.199),
        Point::degrees(36.105, -115.199),
    );
    assert_eq!((more[1].points.len(), ends(&more[1])), (2, column));
    let east = Bounds::degrees(36.0995, -115.1975, 36.1055, -115.1945);
    let cut = sketch::roads(&g, east, 6, 1.0).unwrap();
    assert_eq!(cut.len(), 1, "{cut:?}");
    assert_eq!(ends(&cut[0]), (Point::degrees(36.1, -115.198), row.1));
    let away = Bounds::around(&[Point::degrees(36.5, -115.5)], 500.0).unwrap();
    assert!(sketch::roads(&g, away, 13, 1.0).unwrap().is_empty());
    let inside_out = Bounds::degrees(36.2, -115.2, 36.1, -115.1);
    assert!(sketch::roads(&g, inside_out, 13, 1.0).is_err());
    let off = Bounds {
        north: 900_000_001,
        ..all
    };
    assert!(sketch::roads(&g, off, 13, 1.0).is_err());
}

#[test]
fn a_box_around_places_grows_by_its_margin() {
    let point = |lat, lon| Point { lat, lon };
    let (at, other) = (Point::degrees(36.1, -115.2), Point::degrees(36.12, -115.17));
    let around = Bounds::around(&[other, at], 1_000.0).unwrap();
    let tall = geo::meters(at, point(other.lat, at.lon)) + 2_000.0;
    let height = geo::meters(point(around.south, at.lon), point(around.north, at.lon));
    assert!((height - tall).abs() < 0.1, "{height} against {tall}");
    let edge = geo::meters(
        point(around.north, around.west),
        point(around.north, at.lon),
    );
    assert!((edge - 1_000.0).abs() < 0.1, "{edge}");
    assert_eq!(Bounds::around(&[], 1_000.0), None);
    let pole = Bounds::around(&[Point::degrees(89.99, 179.99)], f64::INFINITY).unwrap();
    assert_eq!((pole.north, pole.east), (900_000_000, 1_800_000_000));
    assert_eq!(Bounds::around(&[at], f64::NAN), Bounds::around(&[at], 0.0));
}
