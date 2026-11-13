//! The exact order refuses numbers it cannot compare and lists shorter than its stops, and keeps its labels within a
//! budget, instead of growing without end or panicking.

mod common;

use common::random::Random;
use common::refused::{input, limit};
use common::windows::Windows;
use offline_route_optimizer::order::{self, Costs, Day, Visit};

fn costs<'a>(
    from: &'a [f64],
    to: &'a [f64],
    between: &'a [Vec<f64>],
    minutes: &'a [Vec<f64>],
) -> Costs<'a> {
    Costs {
        from,
        to,
        between,
        minutes,
    }
}

fn windows(open: Vec<f64>, close: Vec<f64>) -> Windows {
    let length = vec![30.0; open.len()];
    Windows {
        open,
        close,
        length,
        held: None,
    }
}

/// A day whose third stop's lateness is not a number.
struct Broken;

impl Day for Broken {
    fn allowed(&self, _: u32, _: usize) -> bool {
        true
    }

    fn visit(&self, _: u32, stop: usize, free: Option<f64>, drive: f64) -> Visit {
        let late = if stop == 2 { f64::NAN } else { 0.0 };
        let free = free.unwrap_or(480.0) + drive + 30.0;
        Visit { late, free }
    }
}

#[test]
fn costs_and_visits_it_cannot_compare_are_refused() {
    let n = 6;
    let day = windows(vec![480.0; n], vec![600.0; n]);
    let (from, to, grid) = (vec![0.0; n], vec![0.0; n], vec![vec![10.0; n]; n]);
    assert!(
        order::best(&day, n, &costs(&from, &to, &grid, &grid))
            .unwrap()
            .is_some()
    );
    let mut nan = from.clone();
    nan[2] = f64::NAN;
    let mut infinite = grid.clone();
    infinite[1][3] = f64::INFINITY;
    let short: Vec<Vec<f64>> = grid.iter().map(|row| row[..n - 1].to_vec()).collect();
    for refused in [
        costs(&nan, &to, &grid, &grid),
        costs(&from, &to, &infinite, &grid),
        costs(&from, &to[..n - 1], &grid, &grid),
        costs(&from, &to, &grid, &short),
    ] {
        assert!(input(order::best(&day, n, &refused)));
    }
    assert!(input(order::best(
        &Broken,
        n,
        &costs(&from, &to, &grid, &grid)
    )));
}

#[test]
fn the_labels_kept_stay_within_their_budget() {
    let (n, mut random) = (8, Random(0x0DDB_A11C_0FFE_E123));
    let open: Vec<f64> = (0..n)
        .map(|_| 480.0 + 15.0 * random.below(32) as f64)
        .collect();
    let close = open.iter().map(|open| open + 60.0).collect();
    let day = windows(open, close);
    let mut row = || (0..n).map(|_| 5.0 + random.below(40) as f64).collect();
    let grid: Vec<Vec<f64>> = (0..n).map(|_| row()).collect();
    let zero = vec![0.0; n];
    let costs = costs(&zero, &zero, &grid, &grid);
    assert!(limit(order::best_within(&day, n, &costs, 10)));
    let found = order::best_within(&day, n, &costs, order::MAX_LABELS).unwrap();
    assert_eq!(found.map(|order| order.len()), Some(n));
}
