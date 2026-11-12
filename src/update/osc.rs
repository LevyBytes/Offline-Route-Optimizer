//! An osmChange file (https://wiki.openstreetmap.org/wiki/OsmChange): created, modified and deleted nodes, ways and
//! relations, each with its tags, a way's points and a relation's members. Read into one set of changes by kind
//! and id, a later change replacing an earlier one, so several days read in order give the result of them all.

use std::collections::BTreeMap;

use super::tags::{BAD, get, kind, number, opened};
use super::xml::{Event, Xml};
use crate::Result;
use crate::osm::{Element, Kind, Member};

/// Each changed element by kind and id: its new contents, or None where it was deleted.
pub type Changes = BTreeMap<(Kind, i64), Option<Element>>;

/// Add one file's changes to `changes`.
pub fn read(text: &str, changes: &mut Changes) -> Result<()> {
    let (mut xml, mut deleting, mut current) = (Xml::new(text), false, None::<Element>);
    while let Some(event) = xml.next()? {
        let (name, attributes, closed) = match event {
            Event::Open(name, attributes, closed) => (name, attributes, closed),
            Event::Close(name) => {
                if matches!(name, "node" | "way" | "relation") {
                    let element = current.take().ok_or(BAD)?;
                    let key = (element.kind(), element.id());
                    changes.insert(key, (!deleting).then_some(element));
                }
                continue;
            }
        };
        match (name, current.as_mut()) {
            ("create" | "modify" | "delete", None) => deleting = name == "delete",
            ("node" | "way" | "relation", None) => {
                let id = number(&attributes, "id")?;
                let element = opened(name, &attributes, id, deleting)?;
                if closed {
                    changes.insert((element.kind(), id), (!deleting).then_some(element));
                } else {
                    current = Some(element);
                }
            }
            ("tag", Some(element)) => {
                let tag = (
                    get(&attributes, "k")?.to_owned(),
                    get(&attributes, "v")?.to_owned(),
                );
                match element {
                    Element::Node(node) => node.tags.push(tag),
                    Element::Way(way) => way.tags.push(tag),
                    Element::Relation(relation) => relation.tags.push(tag),
                }
            }
            ("nd", Some(Element::Way(way))) => way.refs.push(number(&attributes, "ref")?),
            ("member", Some(Element::Relation(relation))) => relation.members.push(Member {
                kind: kind(get(&attributes, "type")?)?,
                id: number(&attributes, "ref")?,
                role: get(&attributes, "role").unwrap_or("").to_owned(),
            }),
            _ => {}
        }
    }
    Ok(())
}
