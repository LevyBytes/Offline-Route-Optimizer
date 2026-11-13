//! An osmChange file (<https://wiki.openstreetmap.org/wiki/OsmChange>): created, modified and deleted nodes, ways and
//! relations, each with its tags, a way's points and a relation's members. Read into one set of changes by kind
//! and id, a later change replacing an earlier one, so several days read in order give the result of them all. The
//! file is one osmChange element holding create, modify and delete blocks, every tag closed by its own name, and an
//! element past OpenStreetMap's own limits is refused.

use std::collections::BTreeMap;

use super::tags::{BAD, get, kind, number, opened, text};
use super::xml::{Event, Xml};
use crate::limits::{MOST_MEMBERS, MOST_REFS};
use crate::osm::{Element, Kind, Member};
use crate::{Error, Result};

/// Each changed element by kind and id: its new contents, or None where it was deleted.
pub type Changes = BTreeMap<(Kind, i64), Option<Element>>;

const SHAPE: Error = Error::Format("osmChange structure");

/// Whether an action block's elements are deleted, from the names of the open tags: the root, then the block.
fn deleted(open: &[&str]) -> Option<bool> {
    match open {
        ["osmChange", "create" | "modify"] => Some(false),
        ["osmChange", "delete"] => Some(true),
        _ => None,
    }
}

fn put(changes: &mut Changes, element: Element, deleted: bool) {
    let key = (element.kind(), element.id());
    changes.insert(key, (!deleted).then_some(element));
}

/// Add one file's changes to `changes`.
pub fn read(file: &str, changes: &mut Changes) -> Result<()> {
    let (mut xml, mut open, mut current, mut rooted) = (Xml::new(file), Vec::new(), None, false);
    while let Some(event) = xml.next()? {
        let (name, attributes, closed) = match event {
            Event::Open(name, attributes, closed) => (name, attributes, closed),
            Event::Close(name) => {
                if open.pop() != Some(name) {
                    return Err(SHAPE);
                }
                if let (Some(deleted), "node" | "way" | "relation") = (deleted(&open), name) {
                    put(changes, current.take().ok_or(BAD)?, deleted);
                }
                continue;
            }
        };
        if open.is_empty() && (rooted || name != "osmChange") {
            return Err(SHAPE);
        }
        rooted = true;
        match (current.as_mut(), name) {
            (None, "node" | "way" | "relation") if open.len() == 1 => return Err(SHAPE),
            (None, "node" | "way" | "relation") if deleted(&open).is_some() => {
                let deleted = deleted(&open) == Some(true);
                let element = opened(name, &attributes, number(&attributes, "id")?, deleted)?;
                match closed {
                    true => put(changes, element, deleted),
                    false => current = Some(element),
                }
            }
            (Some(element), "tag") if open.len() == 3 => {
                let tag = (text(get(&attributes, "k")?)?, text(get(&attributes, "v")?)?);
                match element {
                    Element::Node(node) => node.tags.push(tag),
                    Element::Way(way) => way.tags.push(tag),
                    Element::Relation(relation) => relation.tags.push(tag),
                }
            }
            (Some(Element::Way(way)), "nd") if open.len() == 3 => {
                if way.refs.len() == MOST_REFS {
                    return Err(Error::Limit("a way of more than 2,000 points"));
                }
                way.refs.push(number(&attributes, "ref")?);
            }
            (Some(Element::Relation(relation)), "member") if open.len() == 3 => {
                if relation.members.len() == MOST_MEMBERS {
                    return Err(Error::Limit("a relation of over 32,000 members"));
                }
                let kind = kind(get(&attributes, "type")?)?;
                let id = number(&attributes, "ref")?;
                let role = text(get(&attributes, "role").unwrap_or(""))?;
                relation.members.push(Member { kind, id, role });
            }
            _ => {}
        }
        if !closed {
            open.push(name);
        }
    }
    match rooted && open.is_empty() {
        true => Ok(()),
        false => Err(Error::Format("osmChange ends inside a tag")),
    }
}
