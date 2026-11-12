//! The labels' order is the best order: on random days with time windows and visit lengths, its lateness and cost
//! equal the best of every order tried one by one, with and without a stop held to its place.

mod common;

use common::random::Random;
use common::windows::{Windows, permutations, score};
use offline_route_optimizer::order::{self, Costs};

#[test]
fn the_order_found_is_the_best() {
    let mut random = Random(0x1234_5678_9ABC_DEF1);
    for round in 0..300 {
        let n = 1 + random.below(7) as usize;
        let open: Vec<f64> = (0..n)
            .map(|_| 480.0 + 15.0 * random.below(32) as f64)
            .collect();
        let close = open
            .iter()
            .map(|open| open + 30.0 + 30.0 * random.below(4) as f64)
            .collect();
        let length = (0..n)
            .map(|_| 30.0 + 15.0 * random.below(4) as f64)
            .collect();
        let held = (round % 3 == 0).then(|| {
            (
                random.below(n as u64) as usize,
                random.below(n as u64) as u32,
            )
        });
        let day = Windows {
            open,
            close,
            length,
            held,
        };
        let minutes: Vec<Vec<f64>> = (0..n)
            .map(|_| (0..n).map(|_| 5.0 + random.below(40) as f64).collect())
            .collect();
        let between = minutes.clone();
        let (from, to): (Vec<f64>, Vec<f64>) = (0..n)
            .map(|_| (random.below(30) as f64, random.below(30) as f64))
            .unzip();
        let costs = Costs {
            from: &from,
            to: &to,
            between: &between,
            minutes: &minutes,
        };
        let best = permutations(n)
            .iter()
            .filter_map(|order| score(&day, &costs, order))
            .reduce(|a, b| {
                if b.0 < a.0 || (b.0 == a.0 && b.1 < a.1) {
                    b
                } else {
                    a
                }
            });
        let found = order::best(&day, n, &costs).unwrap();
        match (best, found) {
            (Some(best), Some(found)) => {
                assert_eq!(
                    score(&day, &costs, &found),
                    Some(best),
                    "round {round}: {found:?}"
                );
            }
            (None, None) => {}
            (best, found) => panic!("round {round}: best {best:?}, found {found:?}"),
        }
    }
}

#[test]
fn more_than_sixteen_stops_are_refused() {
    let day = Windows {
        open: vec![0.0; 17],
        close: vec![0.0; 17],
        length: vec![0.0; 17],
        held: None,
    };
    let rows = vec![vec![0.0; 17]; 17];
    let costs = Costs {
        from: &rows[0],
        to: &rows[0],
        between: &rows,
        minutes: &rows,
    };
    assert!(order::best(&day, 17, &costs).is_err());
}
