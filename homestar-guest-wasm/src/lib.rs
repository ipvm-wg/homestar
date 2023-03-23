wit_bindgen::generate!("test" in "./wits");

struct Component;

type Matrix = Vec<Vec<u8>>;

impl Homestar for Component {
    fn add_one(a: i32) -> i32 {
        a + 1
    }

    fn append_string(a: String) -> String {
        let b = "world";
        [a, b.to_string()].join("\n")
    }

    fn transpose(matrix: Matrix) -> Matrix {
        assert!(!matrix.is_empty());
        let len = matrix[0].len();
        (0..len)
            .map(|i| matrix.iter().map(|row| row[i]).collect())
            .collect()
    }
}

export_homestar!(Component);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_one() {
        assert_eq!(Component::add_one(42), 43);
    }

    #[test]
    fn append_string() {
        assert_eq!(
            Component::append_string("jimmy eat".to_string()),
            "jimmy eat\nworld".to_string()
        );
    }

    #[test]
    fn transpose() {
        let matrix: Matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let transposed = Component::transpose(matrix.clone());
        assert_ne!(transposed, matrix);
        assert_eq!(Component::transpose(transposed), matrix);
    }
}
