//! The box a sketch covers.

use crate::geo;
use crate::osm::Point;

/// A box on the map: its south, west, north and east edges in ten-millionths of a degree, as a Point keeps them. It
/// does not cross the 180th meridian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub south: i32,
    pub west: i32,
    pub north: i32,
    pub east: i32,
}

impl Bounds {
    /// A box from its south, west, north and east edges in degrees.
    pub fn degrees(south: f64, west: f64, north: f64, east: f64) -> Bounds {
        let (corner, other) = (Point::degrees(south, west), Point::degrees(north, east));
        Bounds {
            south: corner.lat,
            west: corner.lon,
            north: other.lat,
            east: other.lon,
        }
    }

    /// The smallest box holding every point, grown by `margin` metres on each side (none without a point). East and
    /// west grow by the margin at the edge nearer a pole, where a degree of longitude is shortest.
    pub fn around(points: &[Point], margin: f64) -> Option<Bounds> {
        let first = points.first()?;
        let (mut south, mut north) = (first.lat, first.lat);
        let (mut west, mut east) = (first.lon, first.lon);
        for point in points {
            (south, north) = (south.min(point.lat), north.max(point.lat));
            (west, east) = (west.min(point.lon), east.max(point.lon));
        }
        let tall = (margin.max(0.0) / geo::RADIUS.to_radians() * 1e7).min(9e8);
        let edge = (f64::from(south.unsigned_abs().max(north.unsigned_abs())) + tall) / 1e7;
        let wide = (tall / edge.min(89.9).to_radians().cos()).min(3.6e9);
        let grow = |value: i32, by: f64, most: f64| {
            (f64::from(value) + by).round().clamp(-most, most) as i32
        };
        Some(Bounds {
            south: grow(south, -tall, 9e8),
            west: grow(west, -wide, 1.8e9),
            north: grow(north, tall, 9e8),
            east: grow(east, wide, 1.8e9),
        })
    }

    /// Whether the box is the right way round and on the earth.
    pub(super) fn usable(&self) -> bool {
        let on = |lat, lon| Point { lat, lon }.valid();
        let round = self.south <= self.north && self.west <= self.east;
        round && on(self.south, self.west) && on(self.north, self.east)
    }
}
