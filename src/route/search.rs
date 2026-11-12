//! One search from a spot. Its states are edges, each reached at its head, kept in a binary heap by cost; reaching a
//! spot is an entry of its own, so a spot is settled the moment its entry leaves the heap. Arrays are kept between
//! searches and only the entries a search touched are reset.

use std::collections::BinaryHeap;

use super::entry::{Entry, Item};
use super::{Goal, LIMIT_METERS, LIMIT_SECONDS, Leg, turn_seconds};
use crate::graph::Graph;
use crate::snap::Spot;

pub struct Search<'a> {
    pub(super) graph: &'a Graph,
    pub(super) goal: Goal,
    pub(super) best: Vec<f64>,
    pub(super) touched: Vec<u32>,
    pub(super) heap: BinaryHeap<Entry>,
    /// The spots being looked for, by chain: (chain, spot), sorted.
    pub(super) on: Vec<(u32, usize)>,
    /// Each whole-degree turn's time, from -180 to 180 (bearings are kept in whole degrees).
    pub(super) turns: Vec<f64>,
}

impl<'a> Search<'a> {
    pub fn new(graph: &'a Graph, goal: Goal) -> Self {
        let best = vec![f64::INFINITY; graph.edges.len()];
        let turns = (-180..=180)
            .map(|angle| turn_seconds(f64::from(angle)))
            .collect();
        Search {
            graph,
            goal,
            best,
            touched: Vec::new(),
            heap: BinaryHeap::new(),
            on: Vec::new(),
            turns,
        }
    }

    /// The legs from one spot to every spot.
    pub fn from(&mut self, spots: &[Spot], from: usize) -> Vec<Option<Leg>> {
        let mut legs = vec![None; spots.len()];
        legs[from] = Some(Leg::default());
        self.on = spots
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != from)
            .map(|(i, s)| (s.chain, i))
            .collect();
        self.on.sort_unstable();
        self.start(spots, from);
        let (mut left, limit) = (spots.len() - 1, self.limit());
        while left > 0 {
            let Some(entry) = self.heap.pop() else { break };
            if entry.key > limit {
                break;
            }
            match entry.item {
                Item::Spot(spot) if legs[spot as usize].is_none() => {
                    legs[spot as usize] = Some(entry.leg);
                    left -= 1;
                }
                Item::Edge(edge) if entry.key <= self.best[edge as usize] => {
                    self.relax(edge, entry, spots)
                }
                _ => {}
            }
        }
        for edge in self.touched.drain(..) {
            self.best[edge as usize] = f64::INFINITY;
        }
        self.heap.clear();
        legs
    }

    fn limit(&self) -> f64 {
        if self.goal == Goal::Time {
            LIMIT_SECONDS
        } else {
            LIMIT_METERS
        }
    }
}
