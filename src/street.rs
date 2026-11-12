//! Street names compared the way an address writes them and the way OpenStreetMap does: "1234 W Sahara Ave, Las
//! Vegas" and "West Sahara Avenue" are the same street. The street is the address's first part without its house
//! number and unit; both names are cut into lowercase words, the suffixes in USPS Publication 28 (Appendix C1) and
//! the directions take one short form each, and the directions are left out, so "S Las Vegas Blvd" matches "Las
//! Vegas Boulevard South"; "Saint" and "Mount" read "St" and "Mt". Ordinals stay as written ("1st", "First").

#[rustfmt::skip]
const SHORT: [(&str, &str); 50] = [
    ("alley", "aly"), ("avenue", "ave"), ("av", "ave"), ("boulevard", "blvd"), ("circle", "cir"), ("court", "ct"),
    ("cove", "cv"), ("crossing", "xing"), ("drive", "dr"), ("expressway", "expy"), ("freeway", "fwy"),
    ("highway", "hwy"), ("lane", "ln"), ("loop", "loop"), ("parkway", "pkwy"), ("pkway", "pkwy"), ("place", "pl"),
    ("plaza", "plz"), ("point", "pt"), ("road", "rd"), ("route", "rte"), ("square", "sq"), ("street", "st"),
    ("str", "st"), ("terrace", "ter"), ("trail", "trl"), ("turnpike", "tpke"), ("way", "way"), ("canyon", "cyn"),
    ("creek", "crk"), ("estates", "ests"), ("heights", "hts"), ("hill", "hl"), ("hills", "hls"), ("mountain", "mtn"),
    ("park", "park"), ("pass", "pass"), ("path", "path"), ("ridge", "rdg"), ("run", "run"), ("summit", "smt"),
    ("valley", "vly"), ("view", "vw"), ("vista", "vis"), ("walk", "walk"), ("bypass", "byp"), ("center", "ctr"),
    ("junction", "jct"), ("saint", "st"), ("mount", "mt"),
];

#[rustfmt::skip]
const DIRECTIONS: [&str; 16] = [
    "n", "s", "e", "w", "ne", "nw", "se", "sw", "north", "south", "east", "west", "northeast", "northwest", "southeast",
    "southwest",
];

const UNITS: [&str; 10] = [
    "apt",
    "apartment",
    "unit",
    "ste",
    "suite",
    "#",
    "spc",
    "space",
    "lot",
    "bldg",
];

/// A street name as a key two spellings of one street share (empty if it holds no word but directions).
pub fn key(name: &str) -> String {
    let lower = name.to_lowercase().replace(['.', ','], " ");
    let words = lower
        .split_whitespace()
        .filter(|word| !DIRECTIONS.contains(word));
    let words: Vec<&str> = words
        .map(|word| {
            SHORT
                .iter()
                .find(|(long, _)| *long == word)
                .map_or(word, |(_, short)| short)
        })
        .collect();
    words.join(" ")
}

/// The street of an address line ("1234 W Sahara Ave Apt 5, Las Vegas, NV" gives "W Sahara Ave"), if it names one.
pub fn street(address: &str) -> Option<String> {
    let first = address.split(',').next()?.trim();
    let mut words: Vec<&str> = first.split_whitespace().collect();
    if words
        .first()
        .is_some_and(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()))
    {
        words.remove(0);
        if words.first().is_some_and(|word| word.contains('/')) {
            words.remove(0);
        }
    }
    if let Some(unit) = words.iter().position(|word| {
        let word = word.to_lowercase();
        UNITS.contains(&word.as_str()) || word.starts_with('#')
    }) {
        words.truncate(unit);
    }
    let street = words.join(" ");
    (!key(&street).is_empty()).then_some(street)
}
