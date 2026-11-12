//! A day's osmChange merged into a file (a node moved, a road deleted, a node, road and restriction added), and an
//! empty day merged into the same file it reads.

mod common;

use common::change::CHANGE;
use common::gzip::gzip;
use common::{Map, temp};
use offline_route_optimizer::osm::{Element, Kind, Point};
use offline_route_optimizer::pbf;
use offline_route_optimizer::update::{self, State};

#[test]
fn a_change_file_merges_into_the_extract() {
    let map = Map::grid(2, &[("highway", "residential")]);
    let (old, new) = (temp("merge-old.osm.pbf"), temp("merge-new.osm.pbf"));
    map.write(&old);
    let changes = update::changes(&[gzip(CHANGE.as_bytes())]).unwrap();
    assert_eq!(changes.len(), 5);
    assert_eq!(changes.get(&(Kind::Way, 11)), Some(&None));
    let state = State {
        sequence: 4322,
        time: 1_794_514_902,
    };
    update::merge(&old, changes, state, &new, &mut |_, _| {}).unwrap();
    let (header, elements) = pbf::elements(&new).unwrap();
    assert_eq!(
        (header.replication_sequence, header.replication_time),
        (Some(4322), Some(1_794_514_902))
    );
    assert_eq!(
        header.replication_url,
        Some("https://download.geofabrik.de/north-america/us/nevada-updates".into())
    );
    let ids: Vec<(Kind, i64)> = elements.iter().map(|e| (e.kind(), e.id())).collect();
    let expected = [
        (Kind::Node, 1000),
        (Kind::Node, 1001),
        (Kind::Node, 1100),
        (Kind::Node, 1101),
        (Kind::Node, 5000),
    ];
    let expected = [
        &expected[..],
        &[
            (Kind::Way, 10),
            (Kind::Way, 50),
            (Kind::Way, 51),
            (Kind::Way, 600),
        ],
    ]
    .concat();
    assert_eq!(ids, [&expected[..], &[(Kind::Relation, 77)]].concat());
    let Element::Node(moved) = &elements[0] else {
        panic!("a node")
    };
    assert_eq!(
        moved.at,
        Point {
            lat: 360_995_000,
            lon: -1_152_000_000
        }
    );
    let Element::Node(added) = &elements[4] else {
        panic!("a node")
    };
    assert_eq!(
        added.tags,
        vec![("name".to_string(), "Jo&Al's \u{2192} \u{a9}".to_string())]
    );
    let Element::Way(road) = &elements[8] else {
        panic!("a way")
    };
    assert_eq!(road.refs, vec![5000, 1000]);
    let Element::Relation(turn) = &elements[9] else {
        panic!("a relation")
    };
    assert_eq!(turn.members.len(), 3);
    update::merge(
        &new,
        update::changes(&[]).unwrap(),
        state,
        &new,
        &mut |_, _| {},
    )
    .unwrap();
    assert_eq!(
        pbf::elements(&new).unwrap().1,
        elements,
        "merging into the same file keeps it whole"
    );
}
