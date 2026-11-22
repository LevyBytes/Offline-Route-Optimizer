//! Lengths and directions on the earth: the haversine formula for lengths (the mean radius, 6,371,008.8 m) and,
//! over the few hundred metres snapping looks at, a flat projection around the place; and a line with fewer points.

use crate::osm::Point;

pub const RADIUS: f64 = 6_371_008.8;

/// The great-circle distance between two points, in metres.
pub fn meters(a: Point, b: Point) -> f64 {
    let (lat1, lat2) = (a.lat_degrees().to_radians(), b.lat_degrees().to_radians());
    let dlat = lat2 - lat1;
    let dlon = (b.lon_degrees() - a.lon_degrees()).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * RADIUS * h.sqrt().min(1.0).asin()
}

/// The direction from a to b, in degrees clockwise from north (0 to 360).
pub fn bearing(a: Point, b: Point) -> f64 {
    let (lat1, lat2) = (a.lat_degrees().to_radians(), b.lat_degrees().to_radians());
    let dlon = (b.lon_degrees() - a.lon_degrees()).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

/// The turn from one direction to another, from -180 (a full turn left) to 180 (right).
pub fn turn(from: f64, to: f64) -> f64 {
    let angle = (to - from + 540.0) % 360.0 - 180.0;
    if angle == -180.0 { 180.0 } else { angle }
}

/// The point of segment a-b closest to p: how far along the segment it is (0 to 1) and how far from p, in metres.
pub fn project(p: Point, a: Point, b: Point) -> (f64, f64) {
    let scale = p.lat_degrees().to_radians().cos();
    let metres = RADIUS.to_radians() / 1e7;
    let xy = |q: Point| {
        (
            (f64::from(q.lon) - f64::from(p.lon)) * scale * metres,
            (f64::from(q.lat) - f64::from(p.lat)) * metres,
        )
    };
    let ((ax, ay), (bx, by)) = (xy(a), xy(b));
    let (dx, dy) = (bx - ax, by - ay);
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (-(ax * dx + ay * dy) / length).clamp(0.0, 1.0)
    };
    let (x, y) = (ax + t * dx, ay + t * dy);
    (t, (x * x + y * y).sqrt())
}

/// The point a fraction of the way from a to b.
pub fn between(a: Point, b: Point, t: f64) -> Point {
    let mix = |p: i32, q: i32| (f64::from(p) + (f64::from(q) - f64::from(p)) * t).round() as i32;
    Point {
        lat: mix(a.lat, b.lat),
        lon: mix(a.lon, b.lon),
    }
}

/// The points of a line with fewer of them, Douglas and Peucker's way (1973): the first and last kept, then, between
/// two kept points, the one farthest from the segment between them (as `project` measures it), while it is more than
/// `tolerance` metres from it (a point on that segment never kept). Iterative, so a long line takes no deep recursion.
pub fn simplify(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let last = points.len() - 1;
    let mut keep = vec![false; points.len()];
    (keep[0], keep[last]) = (true, true);
    let mut spans = vec![(0, last)];
    while let Some((a, b)) = spans.pop() {
        let far = (a + 1..b)
            .map(|k| (project(points[k], points[a], points[b]).1, k))
            .fold(
                (0.0, a),
                |best, next| if next.0 > best.0 { next } else { best },
            );
        if far.1 > a && far.0 > tolerance {
            keep[far.1] = true;
            spans.push((a, far.1));
            spans.push((far.1, b));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(point, kept)| kept.then_some(*point))
        .collect()
}
