#[allow(clippy::all)]
#[rustfmt::skip]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {
    fn add_one(input: i32) -> i32 {
        input + 1
    }

    fn add_two(input: i32) -> i32 {
        input + 2
    }
}

bindings::export!(Component with_types_in bindings);
