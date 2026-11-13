//! What the tests share: a made-up town written as a real PBF file (a grid of streets around Las Vegas's latitude,
//! with whatever one-way streets, closed roads and turn restrictions a test adds), the slow reference search, a day
//! of time windows for the order tests, a gzip writer, a day's change file and five real places; for the limits
//! tests, a hand-made wire format and a refusal's kind.

#![allow(dead_code)]

pub mod change;
pub mod file;
pub mod gzip;
pub mod places;
pub mod random;
pub mod reference;
pub mod refused;
pub mod town;
pub mod windows;
pub mod wire;

pub use file::temp;

use offline_route_optimizer::osm::{Kind, Member, Node, Point, Relation, Way};

#[derive(Default)]
pub struct Map {
    pub nodes: Vec<Node>,
    pub ways: Vec<Way>,
    pub relations: Vec<Relation>,
}

pub fn tags(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

impl Map {
    pub fn node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.push(Node {
            id,
            at: Point::degrees(lat, lon),
            tags: Vec::new(),
        });
    }

    pub fn way(&mut self, id: i64, refs: &[i64], pairs: &[(&str, &str)]) {
        self.ways.push(Way {
            id,
            refs: refs.to_vec(),
            tags: tags(pairs),
        });
    }

    pub fn restriction(&mut self, id: i64, from: i64, via: i64, to: i64, kind: &str) {
        let member = |kind, id, role: &str| Member {
            kind,
            id,
            role: role.to_string(),
        };
        let members = vec![
            member(Kind::Way, from, "from"),
            member(Kind::Node, via, "via"),
            member(Kind::Way, to, "to"),
        ];
        self.relations.push(Relation {
            id,
            members,
            tags: tags(&[("type", "restriction"), ("restriction", kind)]),
        });
    }

    /// A grid of `size` by `size` junctions 0.001 degrees apart; junction (row, column) is node 1000 + 100 row +
    /// column; row r's street is way 10 + r ("W Row r Street"), column c's is way 50 + c ("Column c Avenue").
    pub fn grid(size: i64, road: &[(&str, &str)]) -> Map {
        let mut map = Map::default();
        for r in 0..size {
            for c in 0..size {
                map.node(
                    1000 + 100 * r + c,
                    36.1 + 0.001 * r as f64,
                    -115.2 + 0.001 * c as f64,
                );
            }
        }
        for r in 0..size {
            let refs: Vec<i64> = (0..size).map(|c| 1000 + 100 * r + c).collect();
            let name = format!("W Row {r} Street");
            map.way(10 + r, &refs, &[road, &[("name", name.as_str())]].concat());
        }
        for c in 0..size {
            let refs: Vec<i64> = (0..size).map(|r| 1000 + 100 * r + c).collect();
            let name = format!("Column {c} Avenue");
            map.way(50 + c, &refs, &[road, &[("name", name.as_str())]].concat());
        }
        map
    }
}
