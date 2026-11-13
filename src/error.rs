//! What can go wrong: a file that cannot be read or written, bytes that are not what their format says, a value the
//! caller passed that cannot be used, a limit the library keeps so hostile data cannot exhaust memory, a request the
//! map cannot answer. More kinds may come, so a caller's match keeps a catch-all arm.

use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Reading or writing a file failed.
    Io(std::io::Error),
    /// The bytes break their format (PBF, zlib, gzip, osmChange XML, the graph file); the text says where.
    Format(&'static str),
    /// A graph file written by another version of this library.
    Version(u32),
    /// A value from the caller this library cannot use (a place off the earth's coordinates, a cost that is not a
    /// finite number, lists of the wrong length, a spot from another graph); the text says which.
    Input(&'static str),
    /// Past a limit this library keeps so that hostile or damaged data cannot exhaust memory; the text says which.
    Limit(&'static str),
    /// The map has no road within reach of a place.
    NoRoad,
    /// More stops than an exact order allows (16).
    TooMany(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "file error: {error}"),
            Error::Format(what) => write!(f, "bad data: {what}"),
            Error::Version(found) => write!(f, "graph file version {found} is not this library's"),
            Error::Input(what) => write!(f, "unusable value: {what}"),
            Error::Limit(what) => write!(f, "past this library's limit: {what}"),
            Error::NoRoad => f.write_str("no road near the place"),
            Error::TooMany(count) => write!(f, "{count} stops; an exact order takes at most 16"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

/// This library's results.
pub type Result<T> = std::result::Result<T, Error>;
