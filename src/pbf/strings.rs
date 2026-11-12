//! A block's string table: each text once, by its index (index 0 left empty, as tags' end marker).

use std::collections::HashMap;

/// The texts in the order first used.
#[derive(Default)]
pub struct Strings {
    index: HashMap<String, u64>,
    pub list: Vec<String>,
}

impl Strings {
    /// The text's index in the table, added at its first use.
    pub fn get(&mut self, text: &str) -> u64 {
        if self.list.is_empty() {
            self.list.push(String::new());
        }
        if let Some(index) = self.index.get(text) {
            return *index;
        }
        let index = self.list.len() as u64;
        self.index.insert(text.to_owned(), index);
        self.list.push(text.to_owned());
        index
    }
}
