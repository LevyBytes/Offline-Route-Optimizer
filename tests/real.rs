//! A real extract, when one is named (ORO_EXTRACT, e.g. Geofabrik's nevada-latest.osm.pbf): the graph built, saved
//! and opened again, five places in and around Las Vegas snapped to the streets their addresses name, and the legs
//! between them, every one reached, longer than the straight line and slower than 100 km/h. Run with
//! `cargo test --release --test real -- --ignored --nocapture`.

mod common;

use std::time::Instant;

use common::places::PLACES;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Goal};
use offline_route_optimizer::{build, geo, snap, store};

#[test]
#[ignore]
fn a_real_extract() {
    let Ok(path) = std::env::var("ORO_EXTRACT") else {
        return;
    };
    let started = Instant::now();
    let mut last = None;
    let graph = build::build(path.as_ref(), "nevada", &mut |step, done, total| {
        if last != Some(step) {
            println!(
                "{:>6.1} s  {step:?} ({done} of {total})",
                started.elapsed().as_secs_f64()
            );
            last = Some(step);
        }
    })
    .unwrap();
    println!(
        "{:>6.1} s  built: {} nodes, {} edges, {} chains, {} points, {} names, {} turns banned, {} cells",
        started.elapsed().as_secs_f64(),
        graph.nodes.len(),
        graph.edges.len(),
        graph.chains.len(),
        graph.points.len(),
        graph.names.len(),
        graph.bans.len(),
        graph.grid.keys.len()
    );
    println!("          map data {:?}", graph.meta);
    let file = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("real.graph");
    let saved = Instant::now();
    store::save(&graph, &file).unwrap();
    let size = std::fs::metadata(&file).unwrap().len();
    let opened = Instant::now();
    let again = store::open(&file).unwrap();
    println!(
        "          saved {:.1} MB in {:.1} s, opened in {:.2} s",
        size as f64 / 1e6,
        (opened - saved).as_secs_f64(),
        opened.elapsed().as_secs_f64()
    );
    assert!(again == graph);
    let spots: Vec<_> = PLACES
        .iter()
        .map(|(name, lat, lon, address)| {
            let spot = snap::snap(&graph, Point::degrees(*lat, *lon), Some(address)).unwrap();
            println!(
                "          {name}: {:?}, {:.0} m off",
                graph.name(spot.chain),
                spot.off
            );
            spot
        })
        .collect();
    for goal in [Goal::Time, Goal::Distance] {
        let timed = Instant::now();
        let table = route::table(&graph, &spots, goal).unwrap();
        println!(
            "          {goal:?} table in {:.3} s",
            timed.elapsed().as_secs_f64()
        );
        for (i, row) in table.iter().enumerate() {
            for (j, leg) in row.iter().enumerate().filter(|(j, _)| *j != i) {
                let leg = leg.unwrap();
                let straight = geo::meters(spots[i].at, spots[j].at);
                println!(
                    "          {} -> {}: {:.1} min, {:.1} km (straight {:.1} km)",
                    PLACES[i].0,
                    PLACES[j].0,
                    leg.seconds / 60.0,
                    leg.meters / 1000.0,
                    straight / 1000.0
                );
                assert!(leg.meters >= straight && leg.meters / leg.seconds < 100.0 / 3.6);
            }
        }
    }
}
