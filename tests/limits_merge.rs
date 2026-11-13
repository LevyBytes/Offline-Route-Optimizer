//! A merge refuses a state older than the extract's (the same state again changes nothing), an extract with a second
//! header block, and a file cut short, and leaves no part file behind when it fails.

mod common;

use std::path::Path;

use common::file::parts;
use common::refused::{format, input};
use common::{Map, temp};
use offline_route_optimizer::build;
use offline_route_optimizer::update::{self, State};

#[test]
fn a_merge_refuses_an_older_state_and_a_second_header() {
    let (old, new) = (temp("older-old.osm.pbf"), temp("older-new.osm.pbf"));
    Map::grid(2, &[("highway", "residential")]).write(&old);
    let merge = |from: &Path, sequence: i64| {
        let time = 1_794_428_502 + 86_400 * (sequence - 4321).max(0);
        let changes = update::changes(&[]).unwrap();
        update::merge(
            from,
            changes,
            State { sequence, time },
            &new,
            &mut |_, _| {},
        )
    };
    assert!(input(merge(&old, 4320)));
    assert!(merge(&old, 4321).is_ok() && merge(&old, 4322).is_ok());
    let file = std::fs::read(&old).unwrap();
    let (twice, cut) = (temp("twice.osm.pbf"), temp("cut.osm.pbf"));
    std::fs::write(&twice, [file.clone(), file.clone()].concat()).unwrap();
    std::fs::write(&cut, &file[..file.len() - 10]).unwrap();
    assert!(format(merge(&twice, 4322)));
    assert!(format(build::build(&twice, "test", &mut |_, _, _| {})));
    assert!(merge(&cut, 4322).is_err());
    assert_eq!(parts(&new), 0, "a failed merge leaves no part file");
}
