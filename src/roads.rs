//! Which ways a car may drive, in which directions and how fast, following OSRM's car profile (profiles/car.lua,
//! BSD-2-Clause, read 2026-11-12): its highway classes and their speeds in km/h, a posted maxspeed taken at 80%
//! (its speed_reduction), half speed on alleys, driveways and parking aisles, no area, construction or proposed road,
//! no reversible way, no private road and none whose access names only other users. Roads open only to reach a
//! destination stay in.

use crate::osm::tag;

/// A drivable way: its class (0 motorway to 13 service), its speed each way in km/h (0: not that way).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Road {
    pub class: u8,
    pub forward: f64,
    pub backward: f64,
}

/// Motorways and their ramps: never where a home is reached from.
pub fn is_highway(class: u8) -> bool {
    class <= 1
}

#[rustfmt::skip]
const CLASSES: [(&str, f64); 14] = [
    ("motorway", 90.0), ("motorway_link", 45.0), ("trunk", 85.0), ("trunk_link", 40.0), ("primary", 65.0),
    ("primary_link", 30.0), ("secondary", 55.0), ("secondary_link", 25.0), ("tertiary", 40.0), ("tertiary_link", 20.0),
    ("unclassified", 25.0), ("residential", 25.0), ("living_street", 10.0), ("service", 15.0),
];

#[rustfmt::skip]
const BLOCKED: [&str; 16] = [
    "no", "agricultural", "forestry", "emergency", "psv", "taxi", "share_taxi", "minibus", "bus", "foot",
    "emergency_vehicle", "restricted", "military", "official", "private", "permit",
];

/// The way's road, if a car may drive it.
pub fn road(tags: &[(String, String)]) -> Option<Road> {
    let highway = tag(tags, "highway")?;
    let (index, default) = CLASSES
        .iter()
        .enumerate()
        .find(|(_, (name, _))| *name == highway)?;
    let (class, default) = (index as u8, default.1);
    if tag(tags, "area") == Some("yes") || tag(tags, "impassable") == Some("yes") {
        return None;
    }
    if tag(tags, "construction").is_some_and(|value| !["no", "widening", "minor"].contains(&value))
    {
        return None;
    }
    // The most specific access tag decides: motorcar, then motor_vehicle, vehicle, access.
    let access = ["motorcar", "motor_vehicle", "vehicle", "access"]
        .iter()
        .find_map(|key| tag(tags, key));
    if access.is_some_and(|value| BLOCKED.contains(&value)) {
        return None;
    }
    let mut speed = tag(tags, "maxspeed")
        .and_then(maxspeed)
        .map_or(default, |posted| posted * 0.8);
    if matches!(
        tag(tags, "service"),
        Some("alley" | "parking" | "parking_aisle" | "driveway")
    ) {
        speed *= 0.5;
    }
    let roundabout = matches!(tag(tags, "junction"), Some("roundabout" | "circular"));
    let implied = roundabout || highway == "motorway" || highway == "motorway_link";
    let (forward, backward) = match tag(tags, "oneway") {
        Some("yes" | "true" | "1") => (speed, 0.0),
        Some("-1" | "reverse") => (0.0, speed),
        Some("reversible" | "alternating") => return None,
        Some("no" | "false" | "0") => (speed, speed),
        _ if implied => (speed, 0.0),
        _ => (speed, speed),
    };
    Some(Road {
        class,
        forward,
        backward,
    })
}

/// A posted limit in km/h: "50", "50 km/h", "45 mph"; none for "none", "signals", "walk" and the like.
pub fn maxspeed(value: &str) -> Option<f64> {
    let value = value.trim();
    let digits = value
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(value.len());
    let number: f64 = value[..digits]
        .parse()
        .ok()
        .filter(|speed: &f64| *speed > 0.0)?;
    let unit = value[digits..].trim().to_ascii_lowercase();
    match unit.as_str() {
        "" | "km/h" | "kmh" | "kph" => Some(number),
        "mph" => Some(number * 1.609_344),
        "knots" => Some(number * 1.852),
        _ => None,
    }
}
