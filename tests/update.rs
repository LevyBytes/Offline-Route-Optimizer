//! Bringing an extract up to date: gzip and its check, replication state files and change paths, exact coordinates
//! and the 90 days after which the extract is downloaded whole (merge.rs merges a day's changes into a file).

mod common;

use common::gzip::gzip;
use offline_route_optimizer::update::{self, State};

#[test]
fn gzip_state_paths_and_coordinates() {
    let text = b"<osmChange version=\"0.6\"></osmChange>".repeat(50);
    let mut file = gzip(&text);
    assert_eq!(update::gunzip(&file).unwrap(), text);
    let last = file.len() - 5;
    file[last] ^= 1;
    assert!(update::gunzip(&file).is_err());
    assert_eq!(update::crc32(b"123456789"), 0xCBF4_3926);
    let state = update::state(
        "#Wed Nov 11 20:21:42 UTC 2026\nsequenceNumber=4321\ntimestamp=2026-11-11T20\\:21\\:42Z\n",
    );
    assert_eq!(
        state.unwrap(),
        State {
            sequence: 4321,
            time: 1_794_428_502
        }
    );
    assert_eq!(
        update::seconds("2000-02-29T00:00:00Z").unwrap(),
        951_782_400
    );
    assert_eq!(update::path(4321), "000/004/321.osc.gz");
    assert_eq!(update::state_path(1_234_567), "001/234/567.state.txt");
    assert_eq!(update::e7("36.1234567").unwrap(), 361_234_567);
    assert_eq!(update::e7("-115.2").unwrap(), -1_152_000_000);
    assert!(update::e7("1.12345678").is_err() && update::e7("1e5").is_err());
    assert!(update::needs_full(0, 91 * 86_400) && !update::needs_full(0, 89 * 86_400));
}
