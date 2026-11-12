//! What can go wrong: a file that cannot be read or written, bytes that are not what their format says, a request
//! the map cannot answer.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Reading or writing a file failed.
    Io(std::io::Error),
    /// The bytes break their format (PBF, zlib, gzip, osmChange XML, the graph file); the text says where.
    Format(&'static str),
    /// A graph file written by another version of this library.
    Version(u32),
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
