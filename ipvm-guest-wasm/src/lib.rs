wit_bindgen::generate!("test" in "./wits");

struct Component;

impl Ipvm for Component {
    fn add_one(a: i32) -> i32 {
        a + 1
    }

    fn append_string(a: String) -> String {
        let b = "world";
        [a, b.to_string()].join("\n")
    }
}

export_ipvm!(Component);
