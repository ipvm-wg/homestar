use std::path::Path;
use image::{
    DynamicImage
};


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

    fn blur(data: Vec<u8>, sigma: f32, width: u32, height: u32) -> Vec<u8> {
        let img_buf= image::RgbImage::from_vec(width, height, data).unwrap();

        let blurred = DynamicImage::ImageRgb8(img_buf).blur(sigma);
        blurred.into_bytes()
    }

    fn crop(data: Vec<u8>, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
        let img_buf= image::RgbImage::from_vec(width, height, data).unwrap();

        // Crop the bottom right rectangle
        let cropped = DynamicImage::ImageRgb8(img_buf).crop(x, y, width, height);
        cropped.into_bytes()
    }

    fn grayscale(data: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
        let img_buf= image::RgbImage::from_vec(width, height, data).unwrap();

        let gray = DynamicImage::ImageRgb8(img_buf).grayscale();
        gray.to_rgb8().into_vec()
    }

    fn rotate90(data: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
        let img_buf= image::RgbImage::from_vec(width, height, data).unwrap();

        let rotated = DynamicImage::ImageRgb8(img_buf).rotate90();
        rotated.into_bytes()
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

    #[test]
    fn blur() {
        let img = image::open(&Path::new("./fixtures/synthcat.jpg")).unwrap();
        let (width, height) = (img.width(), img.height());
        let img_vec = img.into_bytes();

        // Call component to blur the image
        let result = Component::blur(img_vec, 10.0, width, height);

        let processed_buf= image::RgbImage::from_vec(width, height, result).unwrap();
        let processed = DynamicImage::ImageRgb8(processed_buf);
        processed.save("./out/blurred.jpg").expect("Failed to write cropped.jpg to filesystem");
    }

    #[test]
    fn crop() {
        let img = image::open(&Path::new("./fixtures/synthcat.jpg")).unwrap();
        let (width, height) = (img.width(), img.height());
        let img_vec = img.into_bytes();

        // Call component to crop the image to the bottom right corner
        let result = Component::crop(img_vec, 200, 200, width, height);

        let processed_buf= image::RgbImage::from_vec(width - 200, height - 200, result).unwrap();
        let processed = DynamicImage::ImageRgb8(processed_buf);
        processed.save("./out/cropped.jpg").expect("Failed to write cropped.jpg to filesystem");
    }

    #[test]
    fn grayscale() {
        let img = image::open(&Path::new("./fixtures/synthcat.jpg")).unwrap();
        let (width, height) = (img.width(), img.height());
        let img_vec = img.into_bytes();

        // Call component to grayscale the image
        let result = Component::grayscale(img_vec, width, height);

        let processed_buf= image::RgbImage::from_vec(width, height, result).unwrap();
        let processed = DynamicImage::ImageRgb8(processed_buf);
        processed.save("./out/graycat.jpg").expect("Failed to write graycat.jpg to filesystem");
    }

    #[test]
    fn rotate() {
        let img = image::open(&Path::new("./fixtures/synthcat.jpg")).unwrap();
        let (width, height) = (img.width(), img.height());
        let img_vec = img.into_bytes();

        // Call component to rotate the image 90 deg clockwise
        let result = Component::rotate90(img_vec, width, height);

        let processed_buf= image::RgbImage::from_vec(width, height, result).unwrap();
        let processed = DynamicImage::ImageRgb8(processed_buf);
        processed.save("./out/rotated.jpg").expect("Failed to write graycat.jpg to filesystem");
    }
}