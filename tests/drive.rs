//! A drive with its road: on made-up towns with random one-way streets, closed roads and turn restrictions, every
//! drive's leg is the one the table finds (None where the table has none), by time and by distance, and its points run
//! from one spot to the other as long as the leg; drives in a row are the drives one by one; one-way streets hold.

mod common;

use common::Map;
use common::random::Random;
use common::town::{graph, town};
use offline_route_optimizer::geo;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Drive, Goal};
use offline_route_optimizer::snap;

/// The length of a drive's points, in metres.
fn length(drive: &Drive) -> f64 {
    drive
        .points
        .windows(2)
        .map(|pair| geo::meters(pair[0], pair[1]))
        .sum()
}

#[test]
fn every_drive_is_its_tables_leg_along_its_road() {
    let mut random = Random(0xD1B5_4A32_D192_ED03);
    for round in 0..8 {
        let g = graph(&town(&mut random), &format!("drive-{round}.osm.pbf"));
        let spots: Vec<_> = (0..6)
            .map(|_| {
                let at = Point::degrees(
                    36.1 + random.below(5000) as f64 * 1e-6,
                    -115.2 + random.below(5000) as f64 * 1e-6,
                );
                snap::snap(&g, at, None).unwrap()
            })
            .collect();
        for goal in [Goal::Time, Goal::Distance] {
            let table = route::table(&g, &spots, goal).unwrap();
            for (i, from) in spots.iter().enumerate() {
                for (j, to) in spots.iter().enumerate() {
                    let drive = route::drive(&g, from, to, goal).unwrap();
                    let name = format!("round {round} {goal:?} {i}->{j}");
                    assert_eq!(drive.as_ref().map(|drive| drive.leg), table[i][j], "{name}");
                    let Some(drive) = drive else { continue };
                    let ends = (drive.points[0], drive.points[drive.points.len() - 1]);
                    assert_eq!(ends, (from.at, to.at), "{name}");
                    assert!(
                        drive.points.windows(2).all(|pair| pair[0] != pair[1]),
                        "{name}"
                    );
                    let slack = 0.1 * drive.points.len() as f64 + 0.01;
                    let off = (length(&drive) - drive.leg.meters).abs();
                    assert!(off <= slack, "{name}: the road {off} m off its leg");
                }
            }
            let row = route::drives(&g, &spots, goal).unwrap();
            assert_eq!(row.len(), spots.len() - 1);
            for (k, drive) in row.into_iter().enumerate() {
                let alone = route::drive(&g, &spots[k], &spots[k + 1], goal).unwrap();
                assert_eq!(drive, alone, "round {round} {goal:?} drive {k}");
            }
        }
    }
}

#[test]
fn a_one_way_street_is_driven_its_way_and_around_the_block_against_it() {
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.ways[0].tags.push(("oneway".into(), "yes".into()));
    let g = graph(&map, "drive-oneway.osm.pbf");
    let west = snap::snap(&g, Point::degrees(36.1, -115.1995), None).unwrap();
    let east = snap::snap(&g, Point::degrees(36.1, -115.1985), None).unwrap();
    let with = route::drive(&g, &west, &east, Goal::Distance)
        .unwrap()
        .unwrap();
    let corner = Point::degrees(36.1, -115.199);
    assert_eq!(with.points, [west.at, corner, east.at]);
    assert!((length(&with) - 90.0).abs() < 1.0, "{}", length(&with));
    let against = route::drive(&g, &east, &west, Goal::Distance)
        .unwrap()
        .unwrap();
    assert!(against.leg.meters > 300.0 && against.points.len() > 4);
    assert!(
        against.points.contains(&Point::degrees(36.101, -115.2)),
        "around the block"
    );
}
