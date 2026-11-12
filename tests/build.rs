//! Building the graph from a file: which roads a car may use and which way, the junctions and chains, the turn
//! restrictions, the file it is saved in, and snapping a place to the road its address names.

mod common;

use common::{Map, temp};
use offline_route_optimizer::graph::{Graph, NONE};
use offline_route_optimizer::osm::Point;
use offline_route_optimizer::{Error, build, snap, store};

fn graph(map: &Map, name: &str) -> Graph {
    let path = temp(name);
    map.write(&path);
    build::build(&path, "test", &mut |_, _, _| {}).unwrap()
}

#[test]
fn roads_junctions_and_directions() {
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.ways[0].tags.push(("oneway".into(), "yes".into()));
    map.ways[1].tags.push(("access".into(), "private".into()));
    let g = graph(&map, "roads.osm.pbf");
    // Rows 0 and 2 and the three columns remain. Row 1 is closed, so each column passes its middle point without
    // a junction there: one chain a column, two a row, and an edge each way except along the one-way row 0.
    assert_eq!(g.chains.len(), 7);
    let one_way = g
        .chains
        .iter()
        .filter(|c| c.forward == NONE || c.backward == NONE)
        .count();
    assert_eq!(one_way, 2);
    assert_eq!(g.edges.len(), 12);
    assert_eq!(g.meta.sequence, 4321);
    assert_eq!(g.names.len(), 5);
    assert!(!g.nodes.contains(&Point::degrees(36.101, -115.199)));
    let corner = g
        .nodes
        .iter()
        .position(|p| *p == Point::degrees(36.1, -115.199))
        .unwrap();
    assert_eq!(g.roads[corner], 3, "row 0 both sides and column 1");
}

#[test]
fn restrictions_and_the_saved_file() {
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.restriction(1, 11, 1101, 51, "no_left_turn");
    map.restriction(2, 50, 1100, 11, "only_right_turn");
    let g = graph(&map, "restrict.osm.pbf");
    assert!(!g.bans.is_empty());
    let via = g
        .nodes
        .iter()
        .position(|p| *p == Point::degrees(36.101, -115.199))
        .unwrap() as u32;
    let row = |e: usize| {
        g.names[g.chains[g.edges[e].chain as usize].name as usize].as_str() == "W Row 1 Street"
    };
    let column = |e: usize| {
        g.names[g.chains[g.edges[e].chain as usize].name as usize].as_str() == "Column 1 Avenue"
    };
    for &(from, to) in &g.bans {
        let (from, to) = (from as usize, to as usize);
        assert!(g.edges[from].head == g.edges[to].tail);
        if g.edges[from].head == via {
            assert!(row(from) && column(to));
        }
    }
    let path = temp("restrict.graph");
    store::save(&g, &path).unwrap();
    assert_eq!(store::open(&path).unwrap(), g);
    assert_eq!(store::meta(&path).unwrap().base_url, g.meta.base_url);
    let mut bytes = std::fs::read(&path).unwrap();
    let at = bytes.len() / 2;
    bytes.truncate(at);
    std::fs::write(&path, &bytes).unwrap();
    assert!(matches!(store::open(&path), Err(Error::Format(_))));
}

#[test]
fn a_place_meets_the_street_its_address_names() {
    let mut map = Map::grid(3, &[("highway", "residential")]);
    map.node(9001, 36.0995, -115.2);
    map.node(9002, 36.0995, -115.198);
    map.way(
        99,
        &[9001, 9002],
        &[("highway", "motorway"), ("name", "Interstate 15")],
    );
    let g = graph(&map, "snap.osm.pbf");
    // 20 m north of Row 0 and 60 m east of Column 0: the address names Column 0 Avenue.
    let place = Point::degrees(36.10018, -115.19933);
    let named = snap::snap(&g, place, Some("12 N Column 0 Ave, Las Vegas, NV")).unwrap();
    assert_eq!(g.name(named.chain), Some("Column 0 Avenue"));
    let nearest = snap::snap(&g, place, None).unwrap();
    assert_eq!(g.name(nearest.chain), Some("W Row 0 Street"));
    assert!((nearest.off - 20.0).abs() < 1.0, "{}", nearest.off);
    let far = Point::degrees(36.2, -115.2);
    assert!(matches!(snap::snap(&g, far, None), Err(Error::NoRoad)));
}
