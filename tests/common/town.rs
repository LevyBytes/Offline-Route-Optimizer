//! Made-up towns built into graphs for the route tests.

use super::random::Random;
use super::{Map, temp};
use offline_route_optimizer::build;
use offline_route_optimizer::graph::Graph;

/// The graph of a made-up town, written and built as a real extract would be.
pub fn graph(map: &Map, name: &str) -> Graph {
    let path = temp(name);
    map.write(&path);
    build::build(&path, "test", &mut |_, _, _| {}).unwrap()
}

/// A 6 by 6 town of random road classes, one-way streets, speed limits and six turn restrictions.
pub fn town(random: &mut Random) -> Map {
    let classes = ["residential", "secondary", "primary", "service", "tertiary"];
    let mut map = Map::grid(6, &[]);
    for way in &mut map.ways {
        let class = classes[random.below(5) as usize];
        way.tags.push(("highway".into(), class.into()));
        match random.below(8) {
            0 => way.tags.push(("oneway".into(), "yes".into())),
            1 => way.tags.push(("oneway".into(), "-1".into())),
            2 => way.tags.push(("maxspeed".into(), "35 mph".into())),
            _ => {}
        }
    }
    for id in 1..=6 {
        let (row, column) = (random.below(6) as i64, random.below(6) as i64);
        let kind = ["no_left_turn", "no_right_turn", "only_straight_on"][random.below(3) as usize];
        map.restriction(id, 10 + row, 1000 + 100 * row + column, 50 + column, kind);
    }
    map
}
