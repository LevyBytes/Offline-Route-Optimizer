//! A search's steps: leaving the start spot each way its road allows, and going on from an edge's head onto each
//! edge leaving there that the turn rules allow, paying for the turn, and reaching the spots on each stretch.

use super::entry::{Entry, Item};
use super::search::Search;
use super::{Goal, Leg, U_TURN_SECONDS};
use crate::geo;
use crate::graph::{Edge, NONE};
use crate::snap::Spot;

impl Search<'_> {
    /// Leaving the spot each way its road allows, and the spots further along the same stretch.
    pub(super) fn start(&mut self, spots: &[Spot], from: usize) {
        let spot = spots[from];
        let chain = self.graph.chains[spot.chain as usize];
        let zero = (0.0, Leg::default());
        for (edge, part) in [
            (chain.forward, 1.0 - spot.share),
            (chain.backward, spot.share),
        ] {
            if edge != NONE {
                let (key, leg) = self.along(zero, edge, part);
                self.reach(edge, key, leg);
            }
        }
        for k in self.on_chain(spot.chain) {
            let (_, other) = self.on[k];
            let share = spots[other].share;
            for (edge, ahead) in [
                (chain.forward, share - spot.share),
                (chain.backward, spot.share - share),
            ] {
                if edge != NONE && ahead >= 0.0 {
                    let (key, leg) = self.along(zero, edge, ahead);
                    self.heap.push(Entry {
                        key,
                        leg,
                        item: Item::Spot(other as u32),
                    });
                }
            }
        }
    }

    /// Every way on from an edge's head: each edge leaving there and each spot on that edge's stretch.
    pub(super) fn relax(&mut self, arriving: u32, entry: Entry, spots: &[Spot]) {
        let g = self.graph;
        let edge = g.edges[arriving as usize];
        let back = |next: &Edge| next.chain == edge.chain && next.forward != edge.forward;
        let open = g
            .out(edge.head)
            .any(|next| !g.banned(arriving, next as u32) && !back(&g.edges[next]));
        for next in g.out(edge.head) {
            let leaving = g.edges[next];
            let u_turn = back(&leaving);
            if g.banned(arriving, next as u32) || (u_turn && open) {
                continue;
            }
            let mut wait = 0.0;
            if g.roads[edge.head as usize] > 2 || u_turn {
                wait = self.turn(geo::turn(
                    f64::from(edge.end_bearing),
                    f64::from(leaving.start_bearing),
                ));
                wait += if u_turn { U_TURN_SECONDS } else { 0.0 };
            }
            let key = entry.key + if self.goal == Goal::Time { wait } else { 0.0 };
            let base = (
                key,
                Leg {
                    seconds: entry.leg.seconds + wait,
                    meters: entry.leg.meters,
                },
            );
            let (full_key, full) = self.along(base, next as u32, 1.0);
            self.reach(next as u32, full_key, full);
            for k in self.on_chain(leaving.chain) {
                let (_, spot) = self.on[k];
                let share = spots[spot].share;
                let (key, leg) = self.along(
                    base,
                    next as u32,
                    if leaving.forward { share } else { 1.0 - share },
                );
                self.heap.push(Entry {
                    key,
                    leg,
                    item: Item::Spot(spot as u32),
                });
            }
        }
    }
}
