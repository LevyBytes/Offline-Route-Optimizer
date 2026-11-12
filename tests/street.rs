//! An address and OpenStreetMap name the same street in different words; the keys they give agree.

use offline_route_optimizer::street::{key, street};

#[test]
fn addresses_and_map_names_agree() {
    assert_eq!(
        street("1234 W Sahara Ave Apt 5, Las Vegas, NV 89102").as_deref(),
        Some("W Sahara Ave")
    );
    assert_eq!(street("1234 1/2 Main St #4").as_deref(), Some("Main St"));
    assert_eq!(street("Apt 5, Henderson"), None);
    assert_eq!(street(""), None);
    assert_eq!(key("W Sahara Ave"), key("West Sahara Avenue"));
    assert_eq!(key("S Las Vegas Blvd."), key("Las Vegas Boulevard South"));
    assert_eq!(key("St Rose Pkwy"), key("Saint Rose Parkway"));
    assert_ne!(key("Sahara Ave"), key("Sahara Dr"));
}
