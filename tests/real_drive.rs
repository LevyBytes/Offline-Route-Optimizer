//! A real extract, when one is named (ORO_EXTRACT, e.g. Geofabrik's nevada-latest.osm.pbf): every drive between five
//! places in and around Las Vegas is the leg the table finds, by time and by distance, along a road as long as the
//! leg; the drives' times, and the sketch of the major roads in a 30 km box around Las Vegas, its lines, points and
//! time. Run with `cargo test --release --test real_drive -- --ignored --nocapture`.

mod common;

use std::time::Instant;

use common::places::PLACES;
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::route::{self, Goal};
use offline_route_optimizer::sketch::{self, Bounds};
use offline_route_optimizer::{build, geo, snap};

#[test]
#[ignore]
fn drives_and_a_sketch_on_a_real_extract() {
    let Ok(path) = std::env::var("ORO_EXTRACT") else {
        return;
    };
    let graph = build::build(path.as_ref(), "nevada", &mut |_, _, _| {}).unwrap();
    let spots: Vec<_> = PLACES
        .iter()
        .map(|(_, lat, lon, address)| {
            snap::snap(&graph, Point::degrees(*lat, *lon), Some(address)).unwrap()
        })
        .collect();
    for goal in [Goal::Time, Goal::Distance] {
        let table = route::table(&graph, &spots, goal).unwrap();
        let mut nearest: Option<(f64, f64, String)> = None;
        for (i, from) in spots.iter().enumerate() {
            for (j, to) in spots.iter().enumerate().filter(|(j, _)| *j != i) {
                let timed = Instant::now();
                let drive = route::drive(&graph, from, to, goal).unwrap().unwrap();
                let ms = timed.elapsed().as_secs_f64() * 1000.0;
                let road: f64 = drive
                    .points
                    .windows(2)
                    .map(|pair| geo::meters(pair[0], pair[1]))
                    .sum();
                let name = format!("{goal:?} {} -> {}", PLACES[i].0, PLACES[j].0);
                println!(
                    "          {name}: {:.1} km, {} points, {ms:.0} ms",
                    drive.leg.meters / 1000.0,
                    drive.points.len()
                );
                assert_eq!(Some(drive.leg), table[i][j], "{name}");
                assert!(
                    (road - drive.leg.meters).abs() <= 0.1 * drive.points.len() as f64,
                    "{name}"
                );
                let off = (drive.leg.meters - 20_000.0).abs();
                if nearest.as_ref().is_none_or(|(best, _, _)| off < *best) {
                    nearest = Some((off, ms, name));
                }
            }
        }
        if let Some((_, ms, name)) = nearest {
            println!("          nearest 20 km: {name}, {ms:.0} ms");
        }
        let timed = Instant::now();
        let row = route::drives(&graph, &spots, goal).unwrap();
        println!(
            "          {goal:?} drives through all five in {:.0} ms",
            timed.elapsed().as_secs_f64() * 1000.0
        );
        let legs: Vec<_> = row
            .iter()
            .map(|drive| drive.as_ref().map(|drive| drive.leg))
            .collect();
        let rows: Vec<_> = (1..spots.len()).map(|j| table[j - 1][j]).collect();
        assert_eq!(legs, rows);
    }
    let middle = Bounds::around(&[Point::degrees(36.12, -115.17)], 15_000.0).unwrap();
    for (most, tolerance) in [(4, 15.0), (4, 30.0), (6, 15.0), (8, 15.0)] {
        let timed = Instant::now();
        let lines = sketch::roads(&graph, middle, most, tolerance).unwrap();
        let ms = timed.elapsed().as_secs_f64() * 1000.0;
        let points: usize = lines.iter().map(|line| line.points.len()).sum();
        println!(
            "          sketch to class {most} at {tolerance} m: {} lines, {points} points in {ms:.0} ms",
            lines.len()
        );
        let link = |class: u8| class <= 9 && class % 2 == 1;
        assert!(!lines.is_empty());
        assert!(
            lines
                .iter()
                .all(|line| line.class <= most && !link(line.class) && line.points.len() >= 2)
        );
    }
}
