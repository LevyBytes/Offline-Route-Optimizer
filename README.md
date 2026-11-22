# Offline Route Optimizer

![Offline Route Optimizer, a Rust library: OpenStreetMap roads, drive times, best stop order. Left, the library's own map of a day in Las Vegas: its sketch of the major roads, the drives in the best order from Downtown Summerlin to Henderson City Hall, and each leg's minutes. Right, the calls that made it and their results.](.github/readme-banner.png)

A Rust library that puts a day's stops in the best order on real roads, on the device, with no server and no paid service. It builds a road graph from an OpenStreetMap extract (Geofabrik's PBF files) and keeps it current from the extract's daily change files instead of downloading it again, taking a fresh extract only after 90 days without an update. It snaps each place to the road its address names, computes drive times and distances between every pair of places with turn costs, one-way streets and turn restrictions, and finds the provably best order from a fixed start to a fixed end with time windows, visits held in their places, and the least lateness before the least driving (Held-Karp with labels, up to 16 stops). For a map of the day, it gives the road each drive takes and a sketch of the area's major roads.

Rust with no unsafe code and one dependency (miniz_oxide, which brings adler2), Apache-2.0. Drive times are averages without traffic; the program showing them credits OpenStreetMap contributors.

## Install

It is not on crates.io; add it from this repository, in Cargo.toml's `[dependencies]`:

```toml
offline_route_optimizer = { git = "https://github.com/LevyBytes/Offline-Route-Optimizer" }
```

## Use

```rust
use offline_route_optimizer::{build, route, sketch, snap, store};
use offline_route_optimizer::osm::Point;

let graph = build::build("nevada-latest.osm.pbf".as_ref(), "nevada", &mut |step, done, total| {
    println!("{step:?} {done} of {total}");
})?;
store::save(&graph, "nevada.graph".as_ref())?;
let graph = store::open("nevada.graph".as_ref())?;
let start = snap::snap(&graph, Point::degrees(36.1086, -115.1424), Some("4505 S Maryland Pkwy, Las Vegas"))?;
let visit = snap::snap(&graph, Point::degrees(36.0306, -114.9817), Some("240 S Water St, Henderson"))?;
let legs = route::table(&graph, &[start, visit], route::Goal::Time)?;
println!("{:.1} minutes", legs[0][1].unwrap().seconds / 60.0);
let drive = route::drive(&graph, &start, &visit, route::Goal::Time)?.unwrap();
println!("{} points along the road", drive.points.len());
let area = sketch::Bounds::around(&[start.at, visit.at], 5_000.0).unwrap();
let roads = sketch::roads(&graph, area, 4, 15.0)?;
println!("{} lines of motorways, trunk and primary roads", roads.len());
```

A point is whole ten-millionths of a degree, latitude first (`Point::lat_degrees` and `lon_degrees` give degrees); a leg is in seconds and metres; a sketch line's class is the road's, from 0 for a motorway to 13 for a service road, the odd ones up to 9 the links a sketch leaves out.

An extract and its change files come from [Geofabrik](https://download.geofabrik.de/): each region's `<name>-latest.osm.pbf`, and beside it `<name>-updates/` with `state.txt` and each day's change at the path `update::path` gives for its sequence number. The program using this library downloads them; the library reads files only.

## What it does

- **Reads OpenStreetMap's PBF format itself** ([src/pbf/](src/pbf/)): the header block with its replication state, dense and plain nodes, ways and relations, zlib-compressed or raw blocks; and writes it, so an extract brought up to date is a PBF file again.
- **Builds the road graph** ([src/build/](src/build/)): the ways a car may drive and which way (OSRM's car profile: its highway classes and speeds, a posted maxspeed at 80%, half speed on alleys, driveways and parking aisles, no private road, none closed to cars, oneway tags and the one-way roundabouts and motorways), junctions where roads meet, chains between them, an edge each way a car may drive, and turn restrictions through a node as forbidden pairs of edges.
- **Saves the graph in its own file and opens it in a moment** ([src/store/](src/store/)): checked on opening, so a damaged file is refused rather than read.
- **Snaps a place to the road it is reached from** ([src/snap.rs](src/snap.rs), [src/street.rs](src/street.rs)): never a motorway or its ramps, and the street its address names when that street is within 300 m, the names compared after USPS Publication 28's suffix abbreviations and with the directions left out ("1234 W Sahara Ave" meets "West Sahara Avenue").
- **Computes the drive between places** ([src/route/](src/route/)): Dijkstra's search over edges rather than junctions, so a turn costs what OSRM's car profile says (up to 7.5 seconds where more than two roads meet, a right turn cheaper than a left) and a forbidden turn is never taken; a U-turn only where a road ends, 20 seconds more; a place's own stretch of road driven in part. One search from each place reaches all the others; four searches run side by side.
- **Traces the road a drive takes** ([src/route/drive.rs](src/route/drive.rs)): the same search from one place to another, stopped where it gets there, each edge remembering the edge it was reached from (four bytes an edge beside the search's eight), then followed back into the road's points in driving order, from the first place's point on its road to the second's. Its time and distance are the ones the table finds for the two places (where two ways cost the same, both take the faster, then the shorter). Drives in a row run four searches side by side.
- **Sketches an area's major roads for a map** ([src/sketch.rs](src/sketch.rs)): the roads up to a class that cross a box, found through the snapping grid rather than by reading every road, ramps and other links left out, each road's stretches (one class and name) joined end to end, cut to the box (a line runs on to its first point past an edge) and given fewer points within a tolerance in metres (Douglas and Peucker's method, `geo::simplify`), motorways first.
- **Orders the stops** ([src/order/](src/order/)): Held-Karp from a start to an end through every stop, with each partial route's lateness, cost and the time the person is free again carried as labels, the exact method for routes with time windows. What a stop's times are, and which stops may come where, is the caller's rule, through the Day trait.
- **Brings a kept extract up to date** ([src/update/](src/update/)): the daily change files (gzip, osmChange XML) read in order, merged into the extract by kind and id, the replication state carried into the new header; past 90 days without an update the caller downloads the extract whole.

## Measured

On Geofabrik's Nevada extract of 2026-11-11 (124 MB) on a desktop PC:

| Step | Result |
| --- | --- |
| Build the graph | 3.1 seconds: 371,159 junctions, 897,899 edges, 3,301 forbidden turns |
| Save and open it | A 65.7 MB file that opens in 0.06 seconds |
| Drive times among five places from Summerlin to Henderson | 0.16 seconds |
| One drive between them with its road | 0.005 to 0.095 seconds (Summerlin to the Bellagio, 19.5 km, in 0.05 seconds, 494 points) |
| The four drives through all five in a row | 0.09 seconds |
| Motorways, trunk and primary roads of a 30 km box around Las Vegas | 0.006 seconds: 520 lines of 2,230 points at a 15 m tolerance (4,734 points with the secondary roads, 8,608 with the tertiary roads too) |
| That day's change file (1,483 changes) merged into the extract | 7.2 seconds; the graph built from the result is the same as the original's |

## What it checks, and what it leaves to the program

Map data is edited by anyone and files can be damaged, so the library refuses or leaves out what honest data never holds rather than run out of memory, hang or panic. `Error::Limit` names a limit it keeps and `Error::Input` a value from the program it cannot use; `Error` may gain kinds, so a match keeps a catch-all arm.

- Extracts: a way of more than 2,000 points or a relation of more than 32,000 members (OpenStreetMap's own limits) is left out. A text longer than 255 characters, numbers that overflow, a varint past 64 bits, a second header block and a file that requires a feature this reader lacks (history, for one) are refused. A block's packed lists, string table and texts have budgets, and a build takes at most 2^28 road points.
- The road graph: a point off the earth counts as missing and splits its road, and so does a piece longer than 50 km or across the 180th meridian, a mistake in the map (none in Nevada is longer than 13 km). A piece is listed in the grid cells it crosses, and the grid holds at most four cells a point. A posted speed outside 5 to 200 km/h is not taken. A turn restriction given twice counts once, and one that would forbid more than 4,096 turns is left out.
- Change files: one osmChange element, its elements inside create, modify and delete blocks, every tag closed by its own name, each attribute with its "="; a way past 2,000 points or a text past 255 characters is refused, and so is more than 128 MiB of text from one file or 512 MiB from all of a call's files (`update::changes_within` takes another limit). A state file's sequence number has one to nine digits and its date a day its month has. A merge refuses a state older than the extract's own (the same state again changes nothing) and writes through a part file of its own beside the destination, removed if the merge fails; a save does the same.
- Graph files: every length is checked against what is left of the file and its kind's limit before room is taken, and every index and place before the graph is used; `store::check` checks a graph the program builds or changes itself the same way.
- Values from the program: `snap::snap` refuses a place off the earth and finds no road on a graph without one. A `Spot` is made only by `snap` and remembers its road, so `route::table`, `route::drive` and `route::drives` refuse one from another graph (as after the day's new map), and a table or a row of drives takes at most 1,000 spots. `sketch::roads` refuses a box turned inside out or off the earth. `order::best` refuses costs that are not finite numbers or are shorter than its stops, and keeps at most `order::MAX_LABELS` labels at once (8,388,608, 192 MiB): sixteen stops bound its work, not its memory, and `order::best_within` takes another budget. A `Day` must answer the same each time it is asked.
- Left to the program: the library makes no request. The updates address (`Meta.base_url`, from the extract's header) and the region name read back from a graph file are whatever wrote the file, so a program that downloads from that address keeps to HTTPS and a host it trusts, and builds no file path from the region name. Change files are not signed (Geofabrik publishes no checksum), so an update's integrity rests on the program's HTTPS, and that the files follow one another without a gap is the program's to see to.

## Checks

`cargo test` runs the checks on made-up towns: the PBF writer and reader agree, the graph keeps the roads, directions and turn restrictions it should, the saved file comes back the same and a damaged one is refused, a place meets the street its address names, every drive equals the one a slow exhaustive search finds (Bellman-Ford over the same edges, by time and by distance), a drive with its road has the leg the table finds and a road from one place to the other as long as the leg, a sketch holds only the classes asked for, joined and cut to its box, a line with fewer points keeps every point it leaves out within its tolerance, the order equals the best of every order tried one by one, a change file merges as it should, and each limit above refuses or leaves out what it should. Three more checks read real files, named by `ORO_EXTRACT` (an extract), `ORO_CHANGE` and `ORO_STATE` (a change file and its state): `cargo test --release --test real --test real_update --test real_drive -- --ignored --nocapture`.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo doc --no-deps` and rust-analyzer's diagnostics report nothing, and `cargo deny check advisories` finds no advisory for the dependency in RustSec's database. Maintained source files are limited to **100 physical lines and 120 characters per line**, with UTF-8 and LF endings. The code has no `unsafe` (`unsafe_code = "forbid"`).

## Left out

- Traffic: drive times are averages at each road's speed, without traffic or traffic lights.
- Turn restrictions through a way (via a way rather than a node), conditional ones, and restrictions for other vehicles only.
- Ferries, toll avoidance, and access that changes with the time of day.

## Contributing

Pull requests are accepted from approved contributors only. To propose a fix or a feature, first open an issue that describes the change your pull request would make, such as a road a drive takes wrongly on a named extract or a limit that refuses honest map data, and the OpenStreetMap or OSRM documentation or the tested behavior it should match. The maintainer reviews the issue and your GitHub account, then approves or declines. New maintainers are approved the same way. See [CONTRIBUTING](.github/CONTRIBUTING.md).

## Map data

OpenStreetMap's data is available under the Open Database License (ODbL) 1.0: [openstreetmap.org/copyright](https://www.openstreetmap.org/copyright). A program that shows drive times made from it credits OpenStreetMap contributors and links to that page.

## Sources

- [The PBF format](https://wiki.openstreetmap.org/wiki/PBF_Format), [osmChange](https://wiki.openstreetmap.org/wiki/OsmChange) and [Geofabrik's downloads and updates](https://download.geofabrik.de/technical.html).
- OSRM's car profile (profiles/car.lua, BSD-2-Clause): the speeds, access values and turn costs this library's car profile follows; no OSRM code is used.
- Dijkstra, "A note on two problems in connexion with graphs", Numerische Mathematik 1, 1959. Held and Karp, "A Dynamic Programming Approach to Sequencing Problems", Journal of SIAM 10(1), 1962. Dumas, Desrosiers, Gelinas and Solomon, "An Optimal Algorithm for the Traveling Salesman Problem with Time Windows", Operations Research 43(2), 1995.
- USPS Publication 28, Appendix C1 (street suffix abbreviations).

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
