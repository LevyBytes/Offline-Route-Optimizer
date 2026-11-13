//! Which kinds of element a read wants, or a block holds.

/// Which kinds of element to read, or a block holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kinds {
    pub nodes: bool,
    pub ways: bool,
    pub relations: bool,
}

impl Kinds {
    pub const ALL: Kinds = Kinds {
        nodes: true,
        ways: true,
        relations: true,
    };
    /// Ways and relations without nodes: a build's first read.
    pub const LINES: Kinds = Kinds {
        nodes: false,
        ways: true,
        relations: true,
    };
}
