//! A day of time windows and visit lengths for the order tests, and the slow way to score every order of it.

use offline_route_optimizer::order::{Costs, Day, Visit};

/// Each stop's window and length.
pub struct Windows {
    pub open: Vec<f64>,
    pub close: Vec<f64>,
    pub length: Vec<f64>,
    /// A stop that must come after exactly this many others.
    pub held: Option<(usize, u32)>,
}

impl Day for Windows {
    fn allowed(&self, before: u32, stop: usize) -> bool {
        match self.held {
            Some((held, place)) => (stop == held) == (before.count_ones() == place),
            None => true,
        }
    }

    fn visit(&self, _before: u32, stop: usize, free: Option<f64>, drive: f64) -> Visit {
        let arrive = free.map_or(self.open[stop], |free| free + drive);
        let start = arrive.max(self.open[stop]);
        Visit {
            late: (arrive - self.close[stop]).max(0.0),
            free: start + self.length[stop],
        }
    }
}

/// An order's lateness and cost worked out step by step, None where it breaks a held place.
pub fn score(day: &Windows, costs: &Costs<'_>, order: &[usize]) -> Option<(f64, f64)> {
    let (mut late, mut cost, mut free, mut before) = (0.0, costs.from[order[0]], None, 0u32);
    for (k, &stop) in order.iter().enumerate() {
        if !day.allowed(before, stop) {
            return None;
        }
        let drive = if k == 0 {
            0.0
        } else {
            costs.minutes[order[k - 1]][stop]
        };
        let visit = day.visit(before, stop, free, drive);
        late += visit.late;
        cost += if k == 0 {
            0.0
        } else {
            costs.between[order[k - 1]][stop]
        };
        (free, before) = (Some(visit.free), before | 1 << stop);
    }
    Some((late, cost + costs.to[order[order.len() - 1]]))
}

/// Every order of n stops.
pub fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![Vec::new()];
    }
    let mut all = Vec::new();
    for shorter in permutations(n - 1) {
        for at in 0..=shorter.len() {
            let mut order = shorter.clone();
            order.insert(at, n - 1);
            all.push(order);
        }
    }
    all
}
