#[allow(clippy::all)]
#[rustfmt::skip]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {
    fn subtract(a: f64, b: f64) -> f64 {
        a - b
    }
}

bindings::export!(Component with_types_in bindings);
