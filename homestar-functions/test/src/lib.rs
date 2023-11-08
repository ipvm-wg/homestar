wit_bindgen::generate!({
    world: "test",
    exports: {
        world: Component,
    }
});

use base64::{engine::general_purpose, Engine};
use std::io::Cursor;

pub struct Component;

type Matrix = Vec<Vec<u16>>;

impl Guest for Component {
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

    fn blur_base64(data: String, sigma: f32) -> Vec<u8> {
        let base64_encoded_png = data.replace("data:image/png;base64,", "");
        let decoded = general_purpose::STANDARD
            .decode(base64_encoded_png)
            .unwrap();
        Self::blur(decoded, sigma)
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

    fn crop_base64(data: String, x: u32, y: u32, target_width: u32, target_height: u32) -> Vec<u8> {
        let base64_encoded_png = data.replace("data:image/png;base64,", "");
        let decoded = general_purpose::STANDARD
            .decode(base64_encoded_png)
            .unwrap();
        Self::crop(decoded, x, y, target_width, target_height)
    }

    fn grayscale(data: Vec<u8>) -> Vec<u8> {
        let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).unwrap();
        let gray = img.grayscale();

        let mut buffer: Vec<u8> = Vec::new();
        gray.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
            .unwrap();

        buffer
    }

    fn grayscale_base64(data: String) -> Vec<u8> {
        let base64_encoded_png = data.replace("data:image/png;base64,", "");
        let decoded = general_purpose::STANDARD
            .decode(base64_encoded_png)
            .unwrap();
        Self::grayscale(decoded)
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

    fn rotate90_base64(data: String) -> Vec<u8> {
        let base64_encoded_png = data.replace("data:image/png;base64,", "");
        let decoded = general_purpose::STANDARD
            .decode(base64_encoded_png)
            .unwrap();
        Self::rotate90(decoded)
    }
}

#[cfg(test)]
mod test_mod {
    use super::*;
    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
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

    #[cfg(feature = "run-image-tests")]
    #[test]
    fn mixed_base64() {
        let img_uri = r#"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABQAAAAUCAYAAACNiR0NAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAG9SURBVHgBrVRLTgJBEK3qaYhLtiZGxxMoN8CdISTiCYSEmLgSTkA8AXFFAgv0BkNMDDu4gXAC2vhhy9IIU2X1CGb4T5CXzCRd3f26Xv0QBOna+ykiVJjBha2A3XhMlbz8vsFsdeCOtP/CAAn4H4bAcKbGMarsgMwiYVUqIj6FHYERXBU2oHV7BYI95HsH6VKk5eUzkw0TPqfDCwK400iGWDXmw+BrJ9mSoE/X59VBZ2/vazjy4xIyzk3tat6Tp8Kh54+d5J8HgRZuhsksWjf7xssfD5npNaxsXvLV9PDz9cGxlSaB7sopA0uQbfQlEeoorAalBvvC5E4IO1KLj0L2ABGQqb+lCLAd8sgsSI5KFtxHXii3GUJxPZWuf5QhIgici7WEwavAKSsFNsB2mCQru5HQFqfW2sAGSLveLuuwBULR7X77fluSlYMVyNQ+LVlx2Z6ec8+TXzOunY5XmK07C1smo3GsTEDFFW/Nls2vBYwtH/G0R9I1gYlUAh04kSzk1g4SuasXjCJZLuWCfVbTg8AEkaAQl3fBViDuKemM0ropExWWg2K6iHYhk8NVMmhF2FazUUiMhKQkXdb9AfsesrssluqmAAAAAElFTkSuQmCC"#;
        // Call component to rotate the image 90 deg clockwise
        let rotated = Component::rotate90_base64(img_uri.into());
        let gray = Component::grayscale(rotated);
        let cropped = Component::crop(gray, 10, 10, 50, 50);
        let result = Component::blur(cropped, 0.1);

        let png_img = image::io::Reader::new(Cursor::new(&result))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
    }
}
