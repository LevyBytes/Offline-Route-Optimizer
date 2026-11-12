//! The labels' search itself: every set of stops in order of its bits, each stop reached last, each label kept there,
//! carried on to each stop the rules allow next; a label beaten on lateness, cost and free time by another at the same
//! state is dropped. The best label at the full set is followed back to the order.

use super::{Costs, Day, MAX_STOPS};
use crate::{Error, Result};

#[derive(Clone, Copy)]
struct Label {
    late: f64,
    cost: f64,
    free: f64,
    last: usize,
    parent: u32,
}

const ROOT: u32 = u32::MAX;

fn beats(a: &Label, b: &Label) -> bool {
    a.late <= b.late && a.cost <= b.cost && a.free <= b.free
}

/// The best order of `count` stops (indexes into the costs), or None if the rules allow none.
pub fn best(day: &impl Day, count: usize, costs: &Costs<'_>) -> Result<Option<Vec<usize>>> {
    if count > MAX_STOPS {
        return Err(Error::TooMany(count));
    }
    let (n, size) = (count, 1usize << count);
    let (mut arena, mut lists): (Vec<Label>, Vec<Vec<u32>>) =
        (Vec::new(), vec![Vec::new(); size * n]);
    for j in (0..n).filter(|j| day.allowed(0, *j)) {
        let visit = day.visit(0, j, None, 0.0);
        lists[(1 << j) * n + j].push(arena.len() as u32);
        arena.push(Label {
            late: visit.late,
            cost: costs.from[j],
            free: visit.free,
            last: j,
            parent: ROOT,
        });
    }
    for mask in 1..size {
        for last in 0..n {
            let here = std::mem::take(&mut lists[mask * n + last]);
            for j in (0..n).filter(|j| mask & (1 << j) == 0 && day.allowed(mask as u32, *j)) {
                let key = (mask | (1 << j)) * n + j;
                for &index in &here {
                    let from = arena[index as usize];
                    let visit = day.visit(mask as u32, j, Some(from.free), costs.minutes[last][j]);
                    let (late, cost) = (from.late + visit.late, from.cost + costs.between[last][j]);
                    let label = Label {
                        late,
                        cost,
                        free: visit.free,
                        last: j,
                        parent: index,
                    };
                    let list = &mut lists[key];
                    if list
                        .iter()
                        .any(|other| beats(&arena[*other as usize], &label))
                    {
                        continue;
                    }
                    list.retain(|other| !beats(&label, &arena[*other as usize]));
                    list.push(arena.len() as u32);
                    arena.push(label);
                }
            }
            lists[mask * n + last] = here;
        }
    }
    let mut best: Option<(u32, f64, f64)> = None;
    for last in 0..n {
        for &index in &lists[(size - 1) * n + last] {
            let label = arena[index as usize];
            let total = label.cost + costs.to[last];
            if best.is_none_or(|(_, late, cost)| {
                label.late < late || label.late == late && total < cost
            }) {
                best = Some((index, label.late, total));
            }
        }
    }
    let Some((mut index, _, _)) = best else {
        return Ok((n == 0).then(Vec::new));
    };
    let mut order = Vec::with_capacity(n);
    while index != ROOT {
        order.push(arena[index as usize].last);
        index = arena[index as usize].parent;
    }
    order.reverse();
    Ok(Some(order))
}
