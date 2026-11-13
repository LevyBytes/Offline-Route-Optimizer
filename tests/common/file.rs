//! The made-up town written as a real PBF file, and a fresh path for each test's files.

use std::path::{Path, PathBuf};

use super::Map;
use offline_route_optimizer::osm::Element;
use offline_route_optimizer::pbf::{Header, Writer};

impl Map {
    pub fn write(&self, path: &Path) {
        let header = Header {
            required: vec!["OsmSchema-V0.6".into(), "DenseNodes".into()],
            optional: vec!["Sort.Type_then_ID".into()],
            replication_time: Some(1_794_428_502),
            replication_sequence: Some(4321),
            replication_url: Some(
                "https://download.geofabrik.de/north-america/us/nevada-updates".into(),
            ),
            ..Header::default()
        };
        let mut writer = Writer::new(std::fs::File::create(path).unwrap(), &header).unwrap();
        let (mut nodes, mut ways, mut relations) = (
            self.nodes.clone(),
            self.ways.clone(),
            self.relations.clone(),
        );
        nodes.sort_by_key(|n| n.id);
        ways.sort_by_key(|w| w.id);
        relations.sort_by_key(|r| r.id);
        nodes
            .into_iter()
            .for_each(|n| writer.add(Element::Node(n)).unwrap());
        ways.into_iter()
            .for_each(|w| writer.add(Element::Way(w)).unwrap());
        relations
            .into_iter()
            .for_each(|r| writer.add(Element::Relation(r)).unwrap());
        writer.finish().unwrap();
    }
}

/// How many part files a save or merge left beside `path`.
pub fn parts(path: &Path) -> usize {
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    let entries = std::fs::read_dir(path.parent().unwrap()).unwrap();
    let names = entries.map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned());
    names
        .filter(|left| left.starts_with(&format!("{name}.")) && left.ends_with(".part"))
        .count()
}

/// A file path in Cargo's folder for test files, made fresh for each test.
pub fn temp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("offline_route_optimizer");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    path
}
