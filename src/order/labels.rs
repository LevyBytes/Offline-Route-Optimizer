//! The labels' search itself: every set of stops in order of its bits, each stop reached last, each label kept there,
//! carried on to each stop the rules allow next; a label beaten on lateness, cost and free time by another at the same
//! state is dropped and takes no more room. The best label at the full set is followed back to the order (back.rs).

use super::back::back;
use super::label::{Label, check, step};
use super::{Costs, Day, MAX_STOPS};
use crate::{Error, Result};

/// The most labels a search keeps at once by default (24 bytes each, so 192 MiB): a day of 16 stops at random times
/// keeps about five million, and a day made to defeat the comparisons could keep billions.
pub const MAX_LABELS: usize = 1 << 23;

/// The best order of `count` stops (indexes into the costs), or None if the rules allow none.
pub fn best(day: &impl Day, count: usize, costs: &Costs<'_>) -> Result<Option<Vec<usize>>> {
    best_within(day, count, costs, MAX_LABELS)
}

/// As `best`, keeping at most `most` labels at once (`Error::Limit` past them).
pub fn best_within(
    day: &impl Day,
    count: usize,
    costs: &Costs<'_>,
    most: usize,
) -> Result<Option<Vec<usize>>> {
    if count > MAX_STOPS {
        return Err(Error::TooMany(count));
    }
    check(costs, count)?;
    let (n, size) = (count, 1usize << count);
    let (mut lists, mut live): (Vec<Vec<Label>>, usize) = (vec![Vec::new(); size * n], 0);
    for j in (0..n).filter(|j| day.allowed(0, *j)) {
        let visit = day.visit(0, j, None, 0.0);
        lists[(1 << j) * n + j].push(Label::new(visit.late, costs.from[j], visit)?);
        live += 1;
    }
    for mask in 1..size {
        for last in 0..n {
            let mut here = std::mem::take(&mut lists[mask * n + last]);
            for j in (0..n).filter(|j| mask & (1 << j) == 0 && day.allowed(mask as u32, *j)) {
                let list = &mut lists[(mask | (1 << j)) * n + j];
                for from in &here {
                    let label = step(day, costs, (mask, last), from, j)?;
                    if list.iter().any(|other| other.beats(&label)) {
                        continue;
                    }
                    let before = list.len();
                    list.retain(|other| !label.beats(other));
                    live = live + 1 - (before - list.len());
                    if live > most {
                        return Err(Error::Limit("an order keeping more labels than allowed"));
                    }
                    list.push(label);
                }
            }
            here.shrink_to_fit();
            lists[mask * n + last] = here;
        }
    }
    let mut best: Option<(usize, Label, f64)> = None;
    for last in 0..n {
        for label in &lists[(size - 1) * n + last] {
            let total = label.cost + costs.to[last];
            if best.is_none_or(|(_, b, cost)| {
                label.late < b.late || label.late == b.late && total < cost
            }) {
                best = Some((last, *label, total));
            }
        }
    }
    let Some((last, label, _)) = best else {
        return Ok((n == 0).then(Vec::new));
    };
    back(day, costs, (&lists, n), (size - 1, last), label).map(Some)
}
