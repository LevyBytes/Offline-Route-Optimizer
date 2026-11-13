//! The PBF reader keeps OpenStreetMap's limits and the format's: a way past 2,000 points and a relation past 32,000
//! members are left out; a text past 255 characters, a tenth varint byte past the 64th bit, a position that overflows
//! and a feature the reader lacks are refused.

mod common;

use common::refused::{format, limit};
use common::wire::{bytes, int, varint, zigzag};
use common::{Map, temp};
use offline_route_optimizer::Error;
use offline_route_optimizer::osm::{Element, Kind, Member, Relation};
use offline_route_optimizer::pbf::{self, Block, Kinds};

#[test]
fn elements_past_openstreetmaps_limits_are_left_out() {
    let mut map = Map::grid(2, &[("highway", "residential")]);
    map.way(90, &[1000; 2_001], &[("highway", "residential")]);
    map.way(91, &[1000; 2_000], &[("highway", "residential")]);
    let role = String::new();
    let member = Member {
        kind: Kind::Node,
        id: 1000,
        role,
    };
    for (id, count) in [(7, 32_001), (8, 32_000)] {
        let (members, tags) = (vec![member.clone(); count], Vec::new());
        map.relations.push(Relation { id, members, tags });
    }
    let path = temp("limits.osm.pbf");
    map.write(&path);
    let (_, elements) = pbf::elements(&path).unwrap();
    let kept: Vec<(Kind, i64)> = elements.iter().map(|e| (e.kind(), e.id())).collect();
    assert!(!kept.contains(&(Kind::Way, 90)) && kept.contains(&(Kind::Way, 91)));
    assert!(!kept.contains(&(Kind::Relation, 7)) && kept.contains(&(Kind::Relation, 8)));
}

/// A block of one node at the raw position given, its string table holding `texts`.
fn block(granularity: u64, texts: &[&[u8]], (lat, lon): (i64, i64)) -> Vec<u8> {
    let (mut table, mut node) = (Vec::new(), Vec::new());
    let (mut group, mut block) = (Vec::new(), Vec::new());
    texts.iter().for_each(|text| bytes(1, text, &mut table));
    int(1, zigzag(1), &mut node);
    int(8, zigzag(lat), &mut node);
    int(9, zigzag(lon), &mut node);
    bytes(1, &node, &mut group);
    bytes(1, &table, &mut block);
    bytes(2, &group, &mut block);
    int(17, granularity, &mut block);
    block
}

fn read(data: &[u8]) -> Result<Vec<Element>, Error> {
    let mut out = Vec::new();
    Block::parse(data)?.elements(Kinds::ALL, &mut out)?;
    Ok(out)
}

#[test]
fn numbers_and_texts_that_break_the_format_are_refused() {
    let plain = read(&block(100, &[b""], (361_000_000, -1_152_000_000)));
    assert_eq!(plain.unwrap().len(), 1);
    assert!(format(read(&block(
        i32::MAX as u64,
        &[b""],
        (i64::MAX / 4, 0)
    ))));
    assert!(format(read(&block(0, &[b""], (0, 0)))));
    assert!(limit(read(&block(100, &[b"", &[b'a'; 1_021]], (0, 0)))));
    assert!(read(&block(100, &[b"", &[b'a'; 1_020]], (0, 0))).is_ok());
    // A replication time whose tenth varint byte holds more than the 64th bit.
    let mut header = Vec::new();
    varint(32 << 3, &mut header);
    header.extend([0xff; 9]);
    header.push(0x02);
    assert!(format(pbf::decode_header(&header)));
    *header.last_mut().unwrap() = 0x01;
    let time = pbf::decode_header(&header).unwrap().replication_time;
    assert_eq!(time, Some(-1));
    // A file that requires history, which this reader does not have.
    let mut features = Vec::new();
    bytes(4, b"OsmSchema-V0.6", &mut features);
    assert!(pbf::decode_header(&features).is_ok());
    bytes(4, b"HistoricalInformation", &mut features);
    assert!(format(pbf::decode_header(&features)));
}
