wit_bindgen::generate!({
    world: "add",
    exports: {
        world: Component,
    }
});

pub struct Component;

impl Guest for Component {
    fn add_two(input: i32) -> i32 {
        input + 2
    }
}
