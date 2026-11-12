//! The PBF writer and reader agree: every element, tag, reference, member and header field written comes back.

mod common;

use common::{Map, tags, temp};
use offline_route_optimizer::osm::{Element, Node, Point};
use offline_route_optimizer::pbf;

#[test]
fn elements_come_back_as_written() {
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.nodes[4].tags = tags(&[("highway", "traffic_signals"), ("name", "Café & Ñ")]);
    map.nodes.push(Node {
        id: 9_000_000_000,
        at: Point {
            lat: -337_000_001,
            lon: 1_512_345_678,
        },
        tags: Vec::new(),
    });
    map.restriction(7, 10, 1001, 51, "no_left_turn");
    let path = temp("roundtrip.osm.pbf");
    map.write(&path);
    let (header, elements) = pbf::elements(&path).unwrap();
    assert_eq!(header.replication_sequence, Some(4321));
    assert_eq!(header.replication_time, Some(1_794_428_502));
    assert_eq!(
        header.replication_url.as_deref(),
        Some("https://download.geofabrik.de/north-america/us/nevada-updates")
    );
    assert_eq!(header.optional, vec!["Sort.Type_then_ID".to_string()]);
    let mut nodes = map.nodes.clone();
    nodes.sort_by_key(|n| n.id);
    let mut expected: Vec<Element> = nodes.into_iter().map(Element::Node).collect();
    expected.extend(map.ways.iter().cloned().map(Element::Way));
    expected.extend(map.relations.iter().cloned().map(Element::Relation));
    assert_eq!(elements, expected);
}

#[test]
fn many_blocks_keep_their_order() {
    let mut map = Map::default();
    for id in 1..=20_000 {
        map.node(id, 36.0 + id as f64 * 1e-6, -115.0);
    }
    let path = temp("blocks.osm.pbf");
    map.write(&path);
    let (_, elements) = pbf::elements(&path).unwrap();
    assert_eq!(elements.len(), 20_000);
    assert!(elements.windows(2).all(|pair| pair[0].id() < pair[1].id()));
    let Element::Node(last) = &elements[19_999] else {
        panic!("a node")
    };
    assert_eq!(last.at, Point::degrees(36.02, -115.0));
}
