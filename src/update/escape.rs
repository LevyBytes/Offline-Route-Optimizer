//! XML's references in attribute values: the five named entities and numeric character references
//! (<https://www.w3.org/TR/xml/>, section 4.1).

use crate::{Error, Result};

const BAD: Error = Error::Format("osmChange XML reference");

/// An attribute value with its references replaced.
pub fn unescape(raw: &str) -> Result<String> {
    let (mut out, mut rest) = (String::with_capacity(raw.len()), raw);
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let end = rest[amp..].find(';').ok_or(BAD)? + amp;
        let name = &rest[amp + 1..end];
        let c = match name {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let code = match name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => name
                        .strip_prefix('#')
                        .and_then(|decimal| decimal.parse().ok()),
                };
                code.and_then(char::from_u32).ok_or(BAD)?
            }
        };
        out.push(c);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
