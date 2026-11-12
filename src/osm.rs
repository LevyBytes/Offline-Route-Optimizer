//! OpenStreetMap's three kinds of element as this library keeps them. Coordinates are whole numbers of ten-millionths
//! of a degree (OpenStreetMap's own precision); a file's metadata (versions, users, changesets) is not kept.

/// A point on the earth: latitude and longitude in ten-millionths of a degree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    pub lat: i32,
    pub lon: i32,
}

impl Point {
    /// A point from degrees.
    pub fn degrees(lat: f64, lon: f64) -> Self {
        Point {
            lat: (lat * 1e7).round() as i32,
            lon: (lon * 1e7).round() as i32,
        }
    }

    pub fn lat_degrees(self) -> f64 {
        f64::from(self.lat) / 1e7
    }

    pub fn lon_degrees(self) -> f64 {
        f64::from(self.lon) / 1e7
    }
}

/// Key and value pairs, in their file order.
pub type Tags = Vec<(String, String)>;

/// The value of a tag, if the element has it.
pub fn tag<'a>(tags: &'a [(String, String)], key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: i64,
    pub at: Point,
    pub tags: Tags,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Way {
    pub id: i64,
    pub refs: Vec<i64>,
    pub tags: Tags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Node,
    Way,
    Relation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub kind: Kind,
    pub id: i64,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub id: i64,
    pub members: Vec<Member>,
    pub tags: Tags,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Node(Node),
    Way(Way),
    Relation(Relation),
}

impl Element {
    pub fn kind(&self) -> Kind {
        match self {
            Element::Node(_) => Kind::Node,
            Element::Way(_) => Kind::Way,
            Element::Relation(_) => Kind::Relation,
        }
    }

    pub fn id(&self) -> i64 {
        match self {
            Element::Node(node) => node.id,
            Element::Way(way) => way.id,
            Element::Relation(relation) => relation.id,
        }
    }
}
