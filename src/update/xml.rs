//! Just enough XML for osmChange files: start and end tags with their attributes, in order; declarations, comments,
//! doctypes and text between tags are passed over. Attribute values keep XML's five named entities and numeric
//! character references (<https://www.w3.org/TR/xml/>, sections 2.4 and 4.1).

use super::escape::unescape;
use crate::{Error, Result};

pub enum Event<'a> {
    /// A start tag: its name, its attributes, and whether it closes itself ("<nd ref="1"/>").
    Open(&'a str, Vec<(&'a str, String)>, bool),
    Close(&'a str),
}

pub struct Xml<'a> {
    text: &'a str,
    at: usize,
}

const BAD: Error = Error::Format("osmChange XML");

impl<'a> Xml<'a> {
    pub fn new(text: &'a str) -> Self {
        Xml { text, at: 0 }
    }

    fn skip_past(&mut self, end: &str) -> Result<()> {
        let found = self.text[self.at..].find(end).ok_or(BAD)?;
        self.at += found + end.len();
        Ok(())
    }

    pub fn next(&mut self) -> Result<Option<Event<'a>>> {
        loop {
            let Some(open) = self.text[self.at..].find('<') else {
                return Ok(None);
            };
            self.at += open;
            let rest = &self.text[self.at..];
            if rest.starts_with("<?") {
                self.skip_past("?>")?;
            } else if rest.starts_with("<!--") {
                self.skip_past("-->")?;
            } else if rest.starts_with("<!") {
                self.skip_past(">")?;
            } else if let Some(rest) = rest.strip_prefix("</") {
                let end = rest.find('>').ok_or(BAD)?;
                self.at += end + 3;
                return Ok(Some(Event::Close(rest[..end].trim())));
            } else {
                return self.open().map(Some);
            }
        }
    }

    fn open(&mut self) -> Result<Event<'a>> {
        let text = self.text;
        let mut at = self.at + 1;
        let end = |text: &str| text.find(|c: char| c.is_whitespace() || "=/>".contains(c));
        let name_end = end(&text[at..]).filter(|end| *end > 0).ok_or(BAD)? + at;
        let (name, mut attributes) = (&text[at..name_end], Vec::new());
        at = name_end;
        loop {
            at += text[at..].len() - text[at..].trim_start().len();
            let rest = &text[at..];
            if rest.starts_with("/>") || rest.starts_with('>') {
                let closed = rest.starts_with('/');
                self.at = at + if closed { 2 } else { 1 };
                return Ok(Event::Open(name, attributes, closed));
            }
            // A name, "=" with spaces allowed around it, then the value in quotes, which holds no "<".
            let key_end = end(rest).filter(|end| *end > 0).ok_or(BAD)?;
            let after = rest[key_end..].trim_start().strip_prefix('=');
            let after = after.ok_or(BAD)?.trim_start();
            let quote = after.chars().next().filter(|c| *c == '"' || *c == '\'');
            let value_start = at + (rest.len() - after.len()) + 1;
            let value_end = text[value_start..].find(quote.ok_or(BAD)?).ok_or(BAD)? + value_start;
            let value = &text[value_start..value_end];
            if value.contains('<') {
                return Err(BAD);
            }
            attributes.push((&rest[..key_end], unescape(value)?));
            at = value_end + 1;
        }
    }
}
