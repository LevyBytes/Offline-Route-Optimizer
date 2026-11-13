//! OpenStreetMap's own limits on an element (<https://wiki.openstreetmap.org/wiki/API_v0.6>), which honest data
//! keeps: past them a file is hostile or damaged.

/// The most points a way holds.
pub(crate) const MOST_REFS: usize = 2_000;
/// The most members a relation holds.
pub(crate) const MOST_MEMBERS: usize = 32_000;
/// The longest key, value or role: 255 characters, each at most four bytes of UTF-8.
pub(crate) const MOST_TEXT: usize = 1_020;
