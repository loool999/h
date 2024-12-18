use image::{GenericImageView, ImageBuffer, Rgb};
use std::collections::HashMap;
use std::path::Path;

fn main() {
    // Path to your input image
    let input_path = Path::new("image1.jpeg"); // Replace with your image

    // Path to the output image with the most used color
    let output_path = Path::new("most_used_color_image.png");

    match analyze_and_create_image(input_path, output_path) {
        Ok(_) => println!(
            "Image with most used color saved to: {}",
            output_path.display()
        ),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn analyze_and_create_image(
    input_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the input image
    let img = image::open(input_path)?;

    // Get the dimensions of the input image
    let (width, height) = img.dimensions();
    println!("Image dimensions: {}x{}", width, height);

    // Convert the image to RGB format
    let img_rgb = img.to_rgb8();

    // Find the most used color
    let most_used_color = find_most_used_color(&img_rgb)?;
    println!("Most used color: R:{}, G:{}, B:{}", most_used_color[0], most_used_color[1], most_used_color[2]);


    // Create a new image with the same size and fill with the most used color
    let new_image = create_image_with_color(width, height, &most_used_color);
    

    // Save the new image
    new_image.save(output_path)?;

    Ok(())
}

fn find_most_used_color(img_rgb: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Rgb<u8>, Box<dyn std::error::Error>> {
    let mut color_counts: HashMap<Rgb<u8>, usize> = HashMap::new();

    // Count the occurrences of each color
    for (_, _, pixel) in img_rgb.enumerate_pixels() {
        *color_counts.entry(*pixel).or_insert(0) += 1;
    }

    // Find the most frequent color
    let most_used = color_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(color, _)| *color)
        .ok_or("No color found in image")?;
    Ok(most_used)
}

fn create_image_with_color(width: u32, height: u32, color: &Rgb<u8>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut new_image = ImageBuffer::new(width, height);
    for (_, _, pixel) in new_image.enumerate_pixels_mut() {
        *pixel = *color;
    }
    new_image
}