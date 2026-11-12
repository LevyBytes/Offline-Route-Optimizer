//! One OSMData block from elements of one kind: nodes as DenseNodes, ways and relations one by one, every string
//! once in the block's table (index 0 left empty, as tags' end marker). Positions keep ten-millionths of a degree
//! (the default granularity, 100 billionths), so nothing is rounded.

use super::out::Out;
use super::strings::Strings;
use crate::osm::{Element, Kind};

pub fn block(elements: &[Element]) -> Vec<u8> {
    let (mut strings, mut group) = (Strings::default(), Out::default());
    strings.get("");
    let nodes: Vec<_> = elements
        .iter()
        .filter_map(|element| {
            if let Element::Node(node) = element {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    if !nodes.is_empty() {
        let mut dense = Out::default();
        dense.deltas(1, nodes.iter().map(|node| node.id));
        dense.deltas(8, nodes.iter().map(|node| i64::from(node.at.lat)));
        dense.deltas(9, nodes.iter().map(|node| i64::from(node.at.lon)));
        if nodes.iter().any(|node| !node.tags.is_empty()) {
            let mut pairs = Vec::new();
            for node in &nodes {
                for (key, value) in &node.tags {
                    pairs.extend([strings.get(key), strings.get(value)]);
                }
                pairs.push(0);
            }
            dense.packed(10, pairs);
        }
        group.bytes(2, &dense.bytes);
    }
    for element in elements {
        let mut item = Out::default();
        let (number, id, tags) = match element {
            Element::Node(_) => continue,
            Element::Way(way) => (3, way.id, &way.tags),
            Element::Relation(relation) => (4, relation.id, &relation.tags),
        };
        item.int(1, id as u64);
        let keys: Vec<u64> = tags.iter().map(|(key, _)| strings.get(key)).collect();
        let vals: Vec<u64> = tags.iter().map(|(_, value)| strings.get(value)).collect();
        item.packed(2, keys);
        item.packed(3, vals);
        match element {
            Element::Way(way) => item.deltas(8, way.refs.iter().copied()),
            Element::Relation(relation) => {
                let roles: Vec<u64> = relation
                    .members
                    .iter()
                    .map(|m| strings.get(&m.role))
                    .collect();
                item.packed(8, roles);
                item.deltas(9, relation.members.iter().map(|member| member.id));
                item.packed(10, relation.members.iter().map(|member| kind(member.kind)));
            }
            Element::Node(_) => {}
        }
        group.bytes(number, &item.bytes);
    }
    let (mut table, mut out) = (Out::default(), Out::default());
    strings
        .list
        .iter()
        .for_each(|text| table.bytes(1, text.as_bytes()));
    out.bytes(1, &table.bytes);
    out.bytes(2, &group.bytes);
    out.bytes
}

fn kind(kind: Kind) -> u64 {
    match kind {
        Kind::Node => 0,
        Kind::Way => 1,
        Kind::Relation => 2,
    }
}
