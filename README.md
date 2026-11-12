# Offline Route Optimizer

Use when a program must put a day's stops in order on real roads with no server and no paid service: Offline Route Optimizer builds a road graph on the device from an OpenStreetMap extract (Geofabrik's PBF files), keeps it current from the extract's daily change files instead of downloading it again, and takes a fresh extract only after 90 days without an update; it snaps each place to the road its address names, computes drive times and distances between every pair of places with turn costs, one-way streets and turn restrictions, and finds the provably best order from a fixed start to a fixed end with time windows, visits held in their places, and the least lateness before the least driving (Held-Karp with labels, up to 16 stops). Rust with no unsafe code and one dependency (miniz_oxide), Apache-2.0. Drive times are averages without traffic; the program showing them credits OpenStreetMap contributors.

## What it does

- Reads OpenStreetMap's PBF format itself (src/pbf/): the header block with its replication state, dense and plain nodes, ways and relations, zlib-compressed or raw blocks; and writes it, so an extract brought up to date is a PBF file again.
- Builds the road graph (src/build/): the ways a car may drive and which way (OSRM's car profile: its highway classes and speeds, a posted maxspeed at 80%, half speed on alleys, driveways and parking aisles, no private road, none closed to cars, oneway tags and the one-way roundabouts and motorways), junctions where roads meet, chains between them, an edge each way a car may drive, and turn restrictions through a node as forbidden pairs of edges.
- Saves the graph in its own file and opens it in a moment (src/store/): checked on opening, so a damaged file is refused rather than read.
- Snaps a place to the road it is reached from (src/snap.rs, src/street.rs): never a motorway or its ramps, and the street its address names when that street is within 300 m, the names compared after USPS Publication 28's suffix abbreviations and with the directions left out ("1234 W Sahara Ave" meets "West Sahara Avenue").
- Computes the drive between places (src/route/): Dijkstra's search over edges rather than junctions, so a turn costs what OSRM's car profile says (up to 7.5 seconds where more than two roads meet, a right turn cheaper than a left) and a forbidden turn is never taken; a U-turn only where a road ends, 20 seconds more; a place's own stretch of road driven in part. One search from each place reaches all the others; four searches run side by side.
- Orders the stops (src/order/): Held-Karp from a start to an end through every stop, with each partial route's lateness, cost and the time the person is free again carried as labels, the exact method for routes with time windows. What a stop's times are, and which stops may come where, is the caller's rule, through the Day trait.
- Brings a kept extract up to date (src/update/): the daily change files (gzip, osmChange XML) read in order, merged into the extract by kind and id, the replication state carried into the new header; past 90 days without an update the caller downloads the extract whole.

## Using it

```rust
use offline_route_optimizer::{build, route, snap, store};
use offline_route_optimizer::osm::Point;

let graph = build::build("nevada-latest.osm.pbf".as_ref(), "nevada", &mut |step, done, total| {
    println!("{step:?} {done} of {total}");
})?;
store::save(&graph, "nevada.graph".as_ref())?;
let graph = store::open("nevada.graph".as_ref())?;
let home = snap::snap(&graph, Point::degrees(36.1086, -115.1424), Some("4505 S Maryland Pkwy, Las Vegas"))?;
let visit = snap::snap(&graph, Point::degrees(36.0306, -114.9817), Some("240 S Water St, Henderson"))?;
let legs = route::table(&graph, &[home, visit], route::Goal::Time);
println!("{:.1} minutes", legs[0][1].unwrap().seconds / 60.0);
```

An extract and its change files come from Geofabrik (https://download.geofabrik.de/): each region's <name>-latest.osm.pbf, and beside it <name>-updates/ with state.txt and each day's change at the path update::path gives for its sequence number. The program using this library downloads them; the library reads files only.

## Measured

On Geofabrik's Nevada extract of 2026-11-11 (124 MB) on a desktop PC: the graph built in 2.9 seconds (371,159 junctions, 897,899 edges, 3,301 forbidden turns) and saved in a 66.8 MB file that opens in 0.06 seconds; the drive times among five places from Summerlin to Henderson in 0.16 seconds; that day's change file (1,483 changes) merged into the extract in 6.8 seconds, the graph built from the result the same as the original's.

## Checks

`cargo test` runs the checks on made-up towns: the PBF writer and reader agree, the graph keeps the roads, directions and turn restrictions it should, the saved file comes back the same and a damaged one is refused, a place meets the street its address names, every drive equals the one a slow exhaustive search finds (Bellman-Ford over the same edges, by time and by distance), the order equals the best of every order tried one by one, and a change file merges as it should. Two more checks read a real extract: ORO_EXTRACT (an extract), ORO_CHANGE and ORO_STATE (a change file and its state) and `cargo test --release --test real --test real_update -- --ignored --nocapture`.

## Left out

- Traffic: drive times are averages at each road's speed, without traffic or traffic lights.
- Turn restrictions through a way (via a way rather than a node), conditional ones, and restrictions for other vehicles only.
- Ferries, toll avoidance, and access that changes with the time of day.

## Map data

OpenStreetMap's data is available under the Open Database License (ODbL) 1.0 (https://www.openstreetmap.org/copyright). A program that shows drive times made from it credits OpenStreetMap contributors and links to that page.

## Sources

- The PBF format: https://wiki.openstreetmap.org/wiki/PBF_Format; osmChange: https://wiki.openstreetmap.org/wiki/OsmChange; Geofabrik's downloads and updates: https://download.geofabrik.de/technical.html.
- OSRM's car profile (profiles/car.lua, BSD-2-Clause): the speeds, access values and turn costs this library's car profile follows; no OSRM code is used.
- Dijkstra, "A note on two problems in connexion with graphs", Numerische Mathematik 1, 1959. Held and Karp, "A Dynamic Programming Approach to Sequencing Problems", Journal of SIAM 10(1), 1962. Dumas, Desrosiers, Gelinas and Solomon, "An Optimal Algorithm for the Traveling Salesman Problem with Time Windows", Operations Research 43(2), 1995.
- USPS Publication 28, Appendix C1 (street suffix abbreviations).

## License

Apache License 2.0 (LICENSE). Copyright 2026 LevyBytes - David Levy (NOTICE).