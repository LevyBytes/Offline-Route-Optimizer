//! A real day's change file merged into a real extract (ORO_EXTRACT, ORO_CHANGE and ORO_STATE: Geofabrik's
//! nevada-latest.osm.pbf, the updates folder's last .osc.gz and its .state.txt). The extract already holds that
//! day, so merging it again changes nothing: the merged file's elements and the graph built from it are the same
//! as the original's. Run with `cargo test --release --test real_update -- --ignored --nocapture`.

use std::time::Instant;

use offline_route_optimizer::{build, pbf, update};

#[test]
#[ignore]
fn a_real_day_merges_into_a_real_extract() {
    let (Ok(extract), Ok(change), Ok(state)) = (
        std::env::var("ORO_EXTRACT"),
        std::env::var("ORO_CHANGE"),
        std::env::var("ORO_STATE"),
    ) else {
        return;
    };
    let state = update::state(&std::fs::read_to_string(state).unwrap()).unwrap();
    let started = Instant::now();
    let changes = update::changes(&[std::fs::read(change).unwrap()]).unwrap();
    let deleted = changes.values().filter(|change| change.is_none()).count();
    println!(
        "{:.2} s: {} changes read, {deleted} of them deletions",
        started.elapsed().as_secs_f64(),
        changes.len()
    );
    let merged = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("merged.osm.pbf");
    let timed = Instant::now();
    update::merge(extract.as_ref(), changes, state, &merged, &mut |_, _| {}).unwrap();
    let size = std::fs::metadata(&merged).unwrap().len();
    println!(
        "{:.1} s: merged, {:.1} MB",
        timed.elapsed().as_secs_f64(),
        size as f64 / 1e6
    );
    let (old_header, old) = pbf::elements(extract.as_ref()).unwrap();
    let (new_header, new) = pbf::elements(&merged).unwrap();
    assert_eq!(new_header.replication_sequence, Some(state.sequence));
    assert_eq!(new_header.replication_url, old_header.replication_url);
    assert_eq!(old.len(), new.len());
    assert!(old == new, "the day was in the extract already");
    let timed = Instant::now();
    let graph = build::build(&merged, "nevada", &mut |_, _, _| {}).unwrap();
    let original = build::build(extract.as_ref(), "nevada", &mut |_, _, _| {}).unwrap();
    println!("{:.1} s: both graphs built", timed.elapsed().as_secs_f64());
    assert!(graph == original);
}
