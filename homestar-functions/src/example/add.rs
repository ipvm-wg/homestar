wit_bindgen::generate!({
    path: "wit/add.wit",
    world: "add",
});

pub struct Component;

impl Add for Component {
    fn add_two(input: i32) -> i32 {
        input + 2
    }
}

export_add!(Component);
