//! A file written beside its destination under a name no other writer shares: made new (never following a link or
//! emptying another's file), renamed over the destination when whole, and removed if dropped before that.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct Part {
    path: PathBuf,
    done: bool,
}

impl Part {
    /// A new, empty file beside `destination`, named after it, the time and a count ("nevada.graph.<time>.4.part").
    pub(crate) fn new(destination: &Path) -> io::Result<(Part, File)> {
        static COUNT: AtomicU64 = AtomicU64::new(0);
        let name = destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let time = SystemTime::now().duration_since(UNIX_EPOCH);
        let time = time.map_or(0, |time| time.as_nanos());
        for _ in 0..64 {
            let count = COUNT.fetch_add(1, Ordering::Relaxed);
            let path = destination.with_file_name(format!("{name}.{time}.{count}.part"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((Part { path, done: false }, file)),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            ErrorKind::AlreadyExists,
            "no free name for a part file",
        ))
    }

    /// The whole file put in its destination's place.
    pub(crate) fn finish(mut self, destination: &Path) -> io::Result<()> {
        fs::rename(&self.path, destination)?;
        self.done = true;
        Ok(())
    }
}

impl Drop for Part {
    fn drop(&mut self) {
        if !self.done {
            let _ = fs::remove_file(&self.path);
        }
    }
}
