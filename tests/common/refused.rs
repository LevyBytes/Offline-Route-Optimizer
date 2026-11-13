//! What kind of refusal a result is, for the limits tests.

use offline_route_optimizer::Error;

pub fn input<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Input(_)))
}

pub fn limit<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Limit(_)))
}

pub fn format<T>(result: Result<T, Error>) -> bool {
    matches!(result, Err(Error::Format(_)))
}
