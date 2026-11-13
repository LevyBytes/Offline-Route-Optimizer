//! The change files' reader refuses what is not one osmChange: a tag left open or closed by another name, an element
//! outside an action block (as after </delete>), an attribute without its "=", another root; what is past the
//! limits: a way of more than 2,000 points, a text past 255 characters, more inflated text than allowed; and state
//! files whose numbers or dates are not plain.

mod common;

use common::gzip::gzip;
use common::refused::limit;
use offline_route_optimizer::Error;
use offline_route_optimizer::update;

fn read(body: &str) -> Result<usize, Error> {
    let text = format!(r#"<osmChange version="0.6">{body}</osmChange>"#);
    update::changes(&[gzip(text.as_bytes())]).map(|changes| changes.len())
}

#[test]
fn a_change_file_must_be_one_well_formed_osmchange() {
    let node = r#"<node id="1" lat="1" lon="1""#;
    assert_eq!(read(&format!("<create>{node}/></create>")).unwrap(), 1);
    for body in [
        format!("<create>{node}></create>"),
        format!("<create>{node}></way></create>"),
        format!(r#"<delete><node id="1"/></delete>{node}/>"#),
        format!(r#"<create><node id="1" lat lon="1"/>{node}/></create>"#),
        r#"<create><node id="1" lat="1><tag k="a" v="b"/></node></create>"#.to_string(),
    ] {
        assert!(read(&body).is_err(), "{body}");
    }
    for text in [
        "<osm><create/></osm>",
        "<osmChange><create>",
        "<osmChange/><osmChange/>",
        "",
    ] {
        assert!(update::changes(&[gzip(text.as_bytes())]).is_err(), "{text}");
    }
}

#[test]
fn a_change_file_keeps_openstreetmaps_limits() {
    let points = |count| r#"<nd ref="1"/>"#.repeat(count);
    let way = |count| format!(r#"<create><way id="1">{}</way></create>"#, points(count));
    assert_eq!(read(&way(2_000)).unwrap(), 1);
    assert!(limit(read(&way(2_001))));
    let tag = |value: &str| {
        format!(r#"<create><node id="1" lat="1" lon="1"><tag k="a" v="{value}"/></node></create>"#)
    };
    assert!(read(&tag(&"é".repeat(510))).is_ok());
    assert!(limit(read(&tag(&"é".repeat(511)))));
    let day = gzip(format!("<osmChange>{}</osmChange>", " ".repeat(1_000)).as_bytes());
    assert!(update::changes_within(std::slice::from_ref(&day), 2_000).is_ok());
    assert!(limit(update::changes_within(&[day.clone(), day], 2_000)));
}

#[test]
fn numbers_and_times_are_read_strictly() {
    assert!(update::e7("999999999999999").is_err() && update::e7("922337203685.4775808").is_err());
    assert!(update::needs_full(i64::MIN, 0) && !update::needs_full(i64::MAX, 0));
    let text =
        |sequence: &str| format!("sequenceNumber={sequence}\ntimestamp=2026-11-11T20\\:21\\:42Z");
    assert_eq!(update::state(&text("4321")).unwrap().sequence, 4321);
    for sequence in ["-1", "+5", "1000000000", " "] {
        assert!(update::state(&text(sequence)).is_err(), "{sequence}");
    }
    let times = [
        "2026-02-29T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-+1-11T20:21:42Z",
    ];
    for time in times {
        assert!(update::seconds(time).is_err(), "{time}");
    }
    assert!(update::seconds("2028-02-29T00:00:00Z").is_ok());
}
