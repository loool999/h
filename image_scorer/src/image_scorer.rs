use image::Rgb;
use std::path::Path;

fn main() {
    // Path to your difference map image
    let input_path = Path::new("difference_map.png");

    match calculate_score(input_path) {
        Ok(score) => println!("Image score: {}", score),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn calculate_score(input_path: &Path) -> Result<f64, Box<dyn std::error::Error>> {
    // Load the image
    let img = image::open(input_path)?;

    // Convert the image to RGB format
    let img_rgb = img.to_rgb8();

    // Initialize the score accumulator and the total pixels
    let mut total_score: f64 = 0.0;
    let mut total_pixels: usize = 0;

    // Go through each pixel and accumulate its score
    for (_, _, pixel) in img_rgb.enumerate_pixels() {
        let pixel_score = calculate_pixel_score(pixel);
        total_score += pixel_score;
        total_pixels += 1;
    }

    // Calculate the average score
    let average_score = if total_pixels > 0 {
        total_score / total_pixels as f64
    } else {
        0.0
    };

    Ok(average_score)
}

// Calculates a score based on how close the colour is to black
fn calculate_pixel_score(pixel: &Rgb<u8>) -> f64 {
    let r = pixel[0] as f64;
    let g = pixel[1] as f64;
    let b = pixel[2] as f64;

    // Calculate the euclidean distance to black
    let score = (r * r + g * g + b * b).sqrt();
    let inverse_score = (255.0 * 1.732 - score)*(100000000000000.0); //1.732 is the aproximation of sqrt(3), and 255 * sqrt(3) is the maximum score.
    //let normalized_score = (inverse_score / (255.0 * 1.732); // Normalize the score to be between 0 and 1

    inverse_score
}