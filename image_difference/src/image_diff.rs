use image::{GenericImageView, ImageBuffer, Rgb};
use std::path::Path;

fn main() {
    // Provide the paths to your images
    let image_path1: &Path = Path::new("image1.jpeg"); // Replace with your image
    let image_path2: &Path = Path::new("image2.png"); // Replace with your image
    let output_path: &Path = Path::new("difference_map.png");

    match compare_images(image_path1, image_path2, output_path) {
        Ok(_) => println!("Image difference map saved to: {}", output_path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn compare_images(
    path1: &Path,
    path2: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the images
    let img1: image::DynamicImage = image::open(path1)?;
    let img2: image::DynamicImage = image::open(path2)?;

    // Ensure that both images have the same dimensions
    if img1.dimensions() != img2.dimensions() {
        return Err("Images have different dimensions".into());
    }

    let (width, height) = img1.dimensions();

    // Convert both images to RGB format for comparison
    let img1_rgb: ImageBuffer<Rgb<u8>, Vec<u8>> = img1.to_rgb8();
    let img2_rgb: ImageBuffer<Rgb<u8>, Vec<u8>> = img2.to_rgb8();

    // Create a new image buffer to store the heatmap
    let mut heatmap: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // Find the maximum color difference to normalize color map
    let mut max_diff: f64 = 0.0;
    for (x, y, pixel1) in img1_rgb.enumerate_pixels() {
        let pixel2: &Rgb<u8> = img2_rgb.get_pixel(x, y);
        let diff: f64 = color_diff(pixel1, pixel2);
        if diff > max_diff {
          max_diff = diff;
        }
    }


    // Generate a color gradient for the heatmap
    let gradient: colorgrad::Gradient = colorgrad::turbo();

    // Calculate the difference between images, creating a heatmap
    for (x, y, pixel1) in img1_rgb.enumerate_pixels() {
        let pixel2: &Rgb<u8> = img2_rgb.get_pixel(x, y);
        let diff: f64 = color_diff(pixel1, pixel2);

        // Normalize the difference value to be between 0.0 and 1.0
        let normalized_diff: f64 = if max_diff > 0.0 { diff / max_diff } else { 0.0 };

        // Get the color from the gradient
        let color: colorgrad::Color = gradient.at(normalized_diff as f64);

        //Convert the gradient color to RGB
        let rgb: Rgb<u8> = Rgb([
          (color.r * 255.0) as u8,
          (color.g * 255.0) as u8,
          (color.b * 255.0) as u8,
        ]);


        heatmap.put_pixel(x, y, rgb);
    }

    // Save the heatmap
    heatmap.save(output_path)?;

    Ok(())
}


fn color_diff(pixel1: &Rgb<u8>, pixel2: &Rgb<u8>) -> f64 {
    let r_diff: f64 = (pixel1[0] as f64 - pixel2[0] as f64).abs();
    let g_diff: f64 = (pixel1[1] as f64 - pixel2[1] as f64).abs();
    let b_diff: f64 = (pixel1[2] as f64 - pixel2[2] as f64).abs();

    (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff).sqrt() //Euclidean distance
}