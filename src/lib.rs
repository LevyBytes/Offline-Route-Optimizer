//! Offline Route Optimizer: the best order for a day's stops on real roads, worked out on the device from an
//! OpenStreetMap extract, with no server. See README.md.

pub mod build;
mod error;
pub mod geo;
pub mod graph;
pub mod grid;
mod limits;
pub mod order;
pub mod osm;
pub mod pbf;
pub mod restrict;
pub mod roads;
pub mod route;
pub mod snap;
pub mod store;
pub mod street;
mod temp;
pub mod update;

pub use error::{Error, Result};
