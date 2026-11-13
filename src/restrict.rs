//! Turn restrictions (relations of type=restriction): from a way, through a node, onto a way. "no_..." forbids that
//! turn; "only_..." forbids every other turn from the "from" way at that node. Left out: restrictions through a way,
//! those that except cars, those for other vehicles only (restriction:hgv and the like) and conditional ones.

use crate::osm::{Kind, Relation, tag};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Restriction {
    pub from: i64,
    pub via: i64,
    pub to: i64,
    /// An "only_..." restriction.
    pub only: bool,
}

pub fn restriction(relation: &Relation) -> Option<Restriction> {
    let tags = &relation.tags;
    if tag(tags, "type") != Some("restriction") {
        return None;
    }
    let value = tag(tags, "restriction:motorcar").or_else(|| tag(tags, "restriction"))?;
    if tag(tags, "except").is_some_and(|except| except.split(';').any(|v| v.trim() == "motorcar")) {
        return None;
    }
    let only = value.starts_with("only_");
    if !only && !value.starts_with("no_") {
        return None;
    }
    let (mut from, mut via, mut to) = (None, None, None);
    for member in &relation.members {
        let slot = match (member.role.as_str(), member.kind) {
            ("from", Kind::Way) => &mut from,
            ("to", Kind::Way) => &mut to,
            ("via", Kind::Node) => &mut via,
            ("via", _) => return None,
            _ => continue,
        };
        if slot.replace(member.id).is_some() {
            return None;
        }
    }
    Some(Restriction {
        from: from?,
        via: via?,
        to: to?,
        only,
    })
}
