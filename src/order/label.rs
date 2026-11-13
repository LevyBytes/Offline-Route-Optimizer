//! One label and one step of the search: a partial route's lateness, cost and free time, the step from it to the next
//! stop, whether one label beats another, and the costs checked before a search begins.

use super::{Costs, Day, Visit};
use crate::{Error, Result};

/// A partial route at its state (the stops visited, the one reached last): how late it is, its cost, when free.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Label {
    pub(super) late: f64,
    pub(super) cost: f64,
    free: f64,
}

impl Label {
    /// A label, refused unless its numbers are finite (one with NaN would never beat another nor be beaten).
    pub(super) fn new(late: f64, cost: f64, visit: Visit) -> Result<Label> {
        let free = visit.free;
        match late.is_finite() && cost.is_finite() && free.is_finite() {
            true => Ok(Label { late, cost, free }),
            false => Err(Error::Input("a visit or cost that is not finite")),
        }
    }

    /// Whether this label is as good as `other` on all three, so `other` need not be kept.
    pub(super) fn beats(&self, other: &Label) -> bool {
        self.late <= other.late && self.cost <= other.cost && self.free <= other.free
    }
}

/// The label one step on: from a label at (stops before, stop reached last) to stop `j`.
pub(super) fn step(
    day: &impl Day,
    costs: &Costs<'_>,
    (mask, last): (usize, usize),
    from: &Label,
    j: usize,
) -> Result<Label> {
    let visit = day.visit(mask as u32, j, Some(from.free), costs.minutes[last][j]);
    Label::new(
        from.late + visit.late,
        from.cost + costs.between[last][j],
        visit,
    )
}

/// Whether every list of costs holds a finite number for each of the `n` stops.
pub(super) fn check(costs: &Costs<'_>, n: usize) -> Result<()> {
    let row = |row: &[f64]| row.len() >= n && row[..n].iter().all(|cost| cost.is_finite());
    let grid = |grid: &[Vec<f64>]| grid.len() >= n && grid[..n].iter().all(|line| row(line));
    match row(costs.from) && row(costs.to) && grid(costs.between) && grid(costs.minutes) {
        true => Ok(()),
        false => Err(Error::Input("costs short of the stops or not finite")),
    }
}
