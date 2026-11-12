//! The parts of an osmChange file's tags: an attribute by name, a number, a coordinate read exactly, a member's kind
//! and an element as its opening tag gives it.

use crate::osm::{Element, Kind, Node, Point, Relation, Way};
use crate::{Error, Result};

pub(super) const BAD: Error = Error::Format("osmChange contents");

pub(super) fn get<'a>(attributes: &'a [(&str, String)], key: &str) -> Result<&'a str> {
    attributes
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, value)| value.as_str())
        .ok_or(BAD)
}

pub(super) fn number(attributes: &[(&str, String)], key: &str) -> Result<i64> {
    get(attributes, key)?.parse().map_err(|_| BAD)
}

/// A coordinate in ten-millionths of a degree, read from its decimal text exactly.
pub fn e7(text: &str) -> Result<i32> {
    let (negative, digits) = text
        .strip_prefix('-')
        .map_or((false, text), |rest| (true, rest));
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let all = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !all(whole) || !all(fraction) || fraction.len() > 7 {
        return Err(BAD);
    }
    let whole: i64 = whole.parse().map_err(|_| BAD)?;
    let fraction: i64 = format!("{fraction:0<7}").parse().map_err(|_| BAD)?;
    let value = whole * 10_000_000 + fraction;
    i32::try_from(if negative { -value } else { value }).map_err(|_| BAD)
}

pub(super) fn kind(name: &str) -> Result<Kind> {
    match name {
        "node" => Ok(Kind::Node),
        "way" => Ok(Kind::Way),
        "relation" => Ok(Kind::Relation),
        _ => Err(BAD),
    }
}

/// A node, way or relation as its opening tag gives it (a deleted node without its place); the tags, points and
/// members that follow fill it in.
pub(super) fn opened(
    name: &str,
    attributes: &[(&str, String)],
    id: i64,
    deleting: bool,
) -> Result<Element> {
    let tags = Vec::new();
    Ok(match name {
        "node" if deleting => Element::Node(Node {
            id,
            at: Point::default(),
            tags,
        }),
        "node" => {
            let at = Point {
                lat: e7(get(attributes, "lat")?)?,
                lon: e7(get(attributes, "lon")?)?,
            };
            Element::Node(Node { id, at, tags })
        }
        "way" => Element::Way(Way {
            id,
            refs: Vec::new(),
            tags,
        }),
        _ => Element::Relation(Relation {
            id,
            members: Vec::new(),
            tags,
        }),
    })
}
