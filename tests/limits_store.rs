//! The graph file refuses a length it does not hold before taking room for it; a save leaves alone another file
//! named like its old part file and leaves none of its own; a graph a program changes is checked as an opened one is.

mod common;

use common::file::parts;
use common::refused::format;
use common::{Map, temp};
use offline_route_optimizer::{build, store};

#[test]
fn a_length_past_the_file_is_refused_before_room_is_taken() {
    let path = temp("short.graph");
    let start = [&b"OROGRAPH"[..], &store::VERSION.to_le_bytes()].concat();
    std::fs::write(&path, [&start[..], &u32::MAX.to_le_bytes()].concat()).unwrap();
    assert!(format(store::meta(&path)) && format(store::open(&path)));
    // A whole graph whose first list claims more records than the file could hold.
    let source = temp("lengths.osm.pbf");
    Map::grid(3, &[("highway", "residential")]).write(&source);
    let graph = build::build(&source, "test", &mut |_, _, _| {}).unwrap();
    let path = temp("lengths.graph");
    store::save(&graph, &path).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let meta = &graph.meta;
    let first = 8 + 4 + 4 + meta.region.len() + 8 + 8 + 4 + meta.base_url.len();
    bytes[first..first + 8].copy_from_slice(&(u64::MAX / 4).to_le_bytes());
    std::fs::write(&path, &bytes).unwrap();
    assert!(format(store::open(&path)));
}

#[test]
fn a_save_keeps_to_its_own_files_and_a_changed_graph_is_checked() {
    let source = temp("stem.osm.pbf");
    Map::grid(3, &[("highway", "residential")]).write(&source);
    let graph = build::build(&source, "test", &mut |_, _, _| {}).unwrap();
    let (path, other) = (temp("stem.graph"), temp("stem.part"));
    std::fs::write(&other, b"another program's file").unwrap();
    store::save(&graph, &path).unwrap();
    assert_eq!(std::fs::read(&other).unwrap(), b"another program's file");
    assert_eq!(store::open(&path).unwrap(), graph);
    assert_eq!(parts(&path), 0);
    assert!(store::check(&graph).is_ok());
    let mut edge = graph.clone();
    edge.chains[0].forward = graph.edges.len() as u32 + 5;
    let mut place = graph.clone();
    place.points[0].lat = 1_000_000_000;
    assert!(store::check(&edge).is_err() && store::check(&place).is_err());
}
