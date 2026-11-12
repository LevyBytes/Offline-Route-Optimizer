//! The best order through a day's stops, exactly: Held-Karp (Bellman 1962; Held and Karp 1962) from a start to an
//! end through every stop, proven best up to 16 stops, with the day's times carried along as labels, the exact
//! method for routes with time windows (Dumas, Desrosiers, Gelinas and Solomon, "An Optimal Algorithm for the
//! Traveling Salesman Problem with Time Windows", Operations Research 43(2), 1995): for each set of stops visited and
//! the stop reached last, every partial route that no other beats on all three of lateness, cost and the time the
//! person is free again. The best order is the least late, then the least cost; ties keep the order found first, so
//! the same day always gives the same answer. What a stop's times are, and which stops may come where, is the
//! caller's rule (`Day`).

mod labels;

pub use labels::best;

/// The most stops an exact order takes (2^16 sets of 16 stops).
pub const MAX_STOPS: usize = 16;

/// A stop's outcome: how late it is and when the person is free again.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Visit {
    pub late: f64,
    pub free: f64,
}

/// The day's rules. `before` is the set of stops already visited (bit i for stop i).
pub trait Day {
    /// Whether the stop may come next.
    fn allowed(&self, before: u32, stop: usize) -> bool;
    /// The stop reached after the ones before: free is when the person can leave the stop before (None at the day's
    /// first stop), drive the minutes between.
    fn visit(&self, before: u32, stop: usize, free: Option<f64>, drive: f64) -> Visit;
}

/// What the order minimizes (`from`, `to` and `between`) and the drive minutes the day's times use.
pub struct Costs<'a> {
    pub from: &'a [f64],
    pub to: &'a [f64],
    pub between: &'a [Vec<f64>],
    pub minutes: &'a [Vec<f64>],
}
