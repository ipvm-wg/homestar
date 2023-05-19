#![allow(clippy::too_many_arguments)]

use std::io::Cursor;
wit_bindgen::generate!("test" in "./wits");

struct Component;

type Matrix = Vec<Vec<u16>>;

impl Homestar for Component {
    fn add_one(a: i32) -> i32 {
        a + 1
    }

    fn append_string(a: String) -> String {
        let b = "world";
        [a, b.to_string()].join("\n")
    }

    fn join_strings(a: String, b: String) -> String {
        [a, b].join("")
    }

    fn transpose(matrix: Matrix) -> Matrix {
        assert!(!matrix.is_empty());
        let len = matrix[0].len();
        (0..len)
            .map(|i| matrix.iter().map(|row| row[i]).collect())
            .collect()
    }

    fn blur(data: Vec<u8>, sigma: f32) -> Vec<u8> {
        let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).unwrap();

        let blurred = img.blur(sigma);

        let mut buffer: Vec<u8> = Vec::new();
        blurred
            .write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        buffer
    }

    fn crop(data: Vec<u8>, x: u32, y: u32, target_width: u32, target_height: u32) -> Vec<u8> {
        let mut img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).unwrap();

        // Crop this image delimited by the bounding rectangle
        let cropped = img.crop(x, y, target_width, target_height);

        let mut buffer: Vec<u8> = Vec::new();
        cropped
            .write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        buffer
    }

    fn grayscale(data: Vec<u8>) -> Vec<u8> {
        let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).unwrap();
        let gray = img.grayscale();

        let mut buffer: Vec<u8> = Vec::new();
        gray.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        buffer
    }

    fn rotate90(data: Vec<u8>) -> Vec<u8> {
        let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).unwrap();

        let rotated = img.rotate90();

        let mut buffer: Vec<u8> = Vec::new();
        rotated
            .write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        buffer
    }
}

export_homestar!(Component);

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

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

    #[test]
    fn blur() {
        let img = image::open(Path::new("./fixtures/synthcat.jpg")).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        // Call component to blur the image
        let result = Component::blur(buffer, 0.3);

        let png_img = image::io::Reader::new(Cursor::new(&result))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        png_img
            .save("./out/blurred.png")
            .expect("Failed to write blurred.png to filesystem");
    }

    #[test]
    fn crop() {
        let img = image::open(Path::new("./fixtures/synthcat.jpg")).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        // Call component to crop the image to a 400x400 square
        let result = Component::crop(buffer, 150, 350, 400, 400);

        let png_img = image::io::Reader::new(Cursor::new(&result))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        png_img
            .save("./out/cropped.png")
            .expect("Failed to write cropped.png to filesystem");
    }

    #[test]
    fn grayscale() {
        let img = image::open(Path::new("./fixtures/synthcat.jpg")).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        // Call component to grayscale the image
        let result = Component::grayscale(buffer);

        let png_img = image::io::Reader::new(Cursor::new(&result))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        png_img
            .save("./out/graycat.png")
            .expect("Failed to write graycat.jpg to filesystem");
    }

    #[test]
    fn rotate() {
        let img = image::open(Path::new("./fixtures/synthcat.jpg")).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        // Call component to rotate the image 90 deg clockwise
        let result = Component::rotate90(buffer);

        let png_img = image::io::Reader::new(Cursor::new(&result))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        png_img
            .save("./out/rotated.png")
            .expect("Failed to write graycat.jpg to filesystem");
    }

    #[test]
    fn mixed() {
        let img = image::open(Path::new("./fixtures/synthcat.jpg")).unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        // Call component to rotate the image 90 deg clockwise
        let rotated = Component::rotate90(buffer);
        let gray = Component::grayscale(rotated);
        let cropped = Component::crop(gray, 150, 350, 400, 400);
        Component::blur(cropped, 0.1);
    }
}
