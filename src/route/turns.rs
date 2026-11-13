//! What a turn costs, as OSRM's car profile times it (profiles/car.lua, read 2026-11-12), driving on the right.

/// OSRM's turn duration in seconds for a turn of `angle` degrees (positive to the right), driving on the right.
pub fn turn_seconds(angle: f64) -> f64 {
    const PENALTY: f64 = 7.5;
    const BIAS: f64 = 1.075;
    let x = if angle >= 0.0 {
        (13.0 / BIAS) * angle / 180.0 - 6.5 * BIAS
    } else {
        (13.0 * BIAS) * -angle / 180.0 - 6.5 / BIAS
    };
    PENALTY / (1.0 + (-x).exp())
}

/// The extra time a U-turn takes on top of its turn.
pub const U_TURN_SECONDS: f64 = 20.0;
