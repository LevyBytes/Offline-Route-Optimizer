//! The best label followed back to its order. The search keeps only its labels, not which label made each: at each
//! state the label in hand was made by the first label one step from which gives it exactly (stops reached last in
//! rising order, then each list in its order, as the search went), since the search keeps the first of equal labels
//! and never a later copy. So `Day` must answer the same each time it is asked.

use super::label::{Label, step};
use super::{Costs, Day};
use crate::{Error, Result};

/// The order of `n` stops that ends with `label` at (stops visited, stop reached last).
pub(super) fn back(
    day: &impl Day,
    costs: &Costs<'_>,
    (lists, n): (&[Vec<Label>], usize),
    (mut mask, mut last): (usize, usize),
    mut label: Label,
) -> Result<Vec<usize>> {
    let mut order = vec![last];
    while mask != 1 << last {
        let before = mask & !(1 << last);
        let mut made = None;
        'search: for prev in (0..n).filter(|prev| before & (1 << prev) != 0) {
            for from in &lists[before * n + prev] {
                if step(day, costs, (before, prev), from, last)? == label {
                    made = Some((prev, *from));
                    break 'search;
                }
            }
        }
        let (prev, from) = made.ok_or(Error::Input("a Day whose answers change"))?;
        order.push(prev);
        (mask, last, label) = (before, prev, from);
    }
    order.reverse();
    Ok(order)
}
