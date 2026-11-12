//! The search finds the least cost drive: on made-up towns with random one-way streets, closed roads and turn
//! restrictions, every leg matches the slow reference search, by time and by distance; and the turn rules hold.

mod common;

use common::random::Random;
use common::town::{graph, town};
use common::{Map, reference};
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Goal, turn_seconds};
use offline_route_optimizer::snap;

#[test]
fn every_leg_matches_the_reference() {
    let mut random = Random(0x9E37_79B9_7F4A_7C15);
    for round in 0..12 {
        let g = graph(&town(&mut random), &format!("town-{round}.osm.pbf"));
        let spots: Vec<_> = (0..7)
            .map(|_| {
                let at = Point::degrees(
                    36.1 + random.below(5000) as f64 * 1e-6,
                    -115.2 + random.below(5000) as f64 * 1e-6,
                );
                snap::snap(&g, at, None).unwrap()
            })
            .collect();
        for goal in [Goal::Time, Goal::Distance] {
            let fast = route::table(&g, &spots, goal);
            let slow = reference::table(&g, &spots, goal);
            for (i, row) in fast.iter().enumerate() {
                for (j, leg) in row.iter().enumerate() {
                    let cost = leg.map(|leg| {
                        if goal == Goal::Time {
                            leg.seconds
                        } else {
                            leg.meters
                        }
                    });
                    let close = match (cost, slow[i][j]) {
                        (Some(a), Some(b)) => (a - b).abs() < 1e-6 * b.max(1.0),
                        (None, None) => true,
                        _ => false,
                    };
                    assert!(
                        close,
                        "round {round} {goal:?} {i}->{j}: {cost:?} against {:?}",
                        slow[i][j]
                    );
                }
            }
        }
    }
}

#[test]
fn right_turns_cost_less_than_left_and_one_way_streets_hold() {
    assert!(turn_seconds(90.0) < turn_seconds(-90.0));
    assert!(turn_seconds(0.0) < 0.1 && (turn_seconds(180.0) - 7.455).abs() < 0.01);
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.ways[0].tags.push(("oneway".into(), "yes".into()));
    let g = graph(&map, "oneway.osm.pbf");
    let west = snap::snap(&g, Point::degrees(36.1, -115.1995), None).unwrap();
    let east = snap::snap(&g, Point::degrees(36.1, -115.1985), None).unwrap();
    let legs = route::table(&g, &[west, east], Goal::Distance);
    let (with, against) = (legs[0][1].unwrap().meters, legs[1][0].unwrap().meters);
    assert!((with - 90.0).abs() < 1.0, "{with}");
    assert!(
        against > 300.0,
        "against the one-way street goes around the block: {against}"
    );
}

#[test]
fn a_dead_end_turns_back() {
    let mut map = Map::default();
    map.node(1, 36.1, -115.2);
    map.node(2, 36.1, -115.199);
    map.way(1, &[1, 2], &[("highway", "residential")]);
    let g = graph(&map, "dead-end.osm.pbf");
    let near = snap::snap(&g, Point::degrees(36.1, -115.1998), None).unwrap();
    let far = snap::snap(&g, Point::degrees(36.1, -115.1992), None).unwrap();
    let legs = route::table(&g, &[near, far], Goal::Time);
    assert!((legs[0][1].unwrap().meters - 54.0).abs() < 1.0);
}
