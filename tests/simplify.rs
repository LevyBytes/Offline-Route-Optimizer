//! A line with fewer points (Douglas and Peucker's way): a straight line keeps only its ends, a zigzag every point
//! farther than the tolerance from its neighbours' line, and on a random walk every point left out lies within the
//! tolerance of the segment between the kept points around it.

mod common;

use common::random::Random;
use offline_route_optimizer::geo;
use offline_route_optimizer::osm::Point;

#[test]
fn a_straight_line_keeps_its_ends_and_a_zigzag_its_corners() {
    let straight: Vec<Point> = (0..=10_000)
        .map(|k| Point::degrees(36.0 + k as f64 * 1e-5, -115.0))
        .collect();
    assert_eq!(
        geo::simplify(&straight, 0.5),
        [straight[0], straight[10_000]]
    );
    let zigzag: Vec<Point> = (0..=20)
        .map(|k| Point::degrees(36.1 + k as f64 * 5e-4, -115.2 + (k % 2) as f64 * 2e-4))
        .collect();
    assert_eq!(geo::simplify(&zigzag, 5.0), zigzag);
    assert_eq!(geo::simplify(&zigzag, 50.0), [zigzag[0], zigzag[20]]);
    assert_eq!(geo::simplify(&zigzag[..2], 50.0), zigzag[..2]);
    assert!(geo::simplify(&[], 1.0).is_empty());
    let same = [straight[0]; 3];
    let kept = geo::simplify(&same, -1.0);
    assert_eq!(
        kept,
        [same[0], same[2]],
        "a point on the segment never kept"
    );
}

#[test]
fn every_point_left_out_lies_within_the_tolerance() {
    let mut random = Random(0x2545_F491_4F6C_DD1D);
    let mut step = |value: i32| value + random.below(1_001) as i32 - 500;
    let mut walk = vec![Point::degrees(36.1, -115.2)];
    for _ in 0..2_000 {
        let last = walk[walk.len() - 1];
        let (lat, lon) = (step(last.lat), step(last.lon));
        walk.push(Point { lat, lon });
    }
    for tolerance in [1.0, 10.0, 100.0] {
        let kept = geo::simplify(&walk, tolerance);
        assert!(kept.len() < walk.len());
        assert_eq!((kept[0], kept[kept.len() - 1]), (walk[0], walk[2_000]));
        let mut at = 0;
        for pair in kept.windows(2) {
            let ahead = walk[at + 1..].iter().position(|point| *point == pair[1]);
            let next = at + 1 + ahead.unwrap();
            for point in &walk[at + 1..next] {
                let off = geo::project(*point, pair[0], pair[1]).1;
                assert!(off <= tolerance, "{off} m off at {tolerance} m");
            }
            at = next;
        }
    }
}
