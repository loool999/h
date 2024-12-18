use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use rand::seq::SliceRandom;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::json;

fn main() {
    // Paths
    let objects_dir = Path::new("objects");
    let saved_dir = Path::new("saved");
    let image2_path = Path::new("image2.png"); //This should exists
    let image1_path = Path::new("image1.jpeg");

    //Ensure directory exist
    if !objects_dir.exists() {
        fs::create_dir(objects_dir).expect("Failed to create objects directory");
    }
    if !saved_dir.exists() {
        fs::create_dir(saved_dir).expect("Failed to create saved directory");
    }
    
    match orchestrate_image_processing(objects_dir, saved_dir, image2_path, image1_path) {
        Ok(_) => println!("Image processing completed successfully."),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn orchestrate_image_processing(
    objects_dir: &Path,
    saved_dir: &Path,
    image2_path: &Path,
    image1_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Select a random image
    let random_image_path = select_random_image(objects_dir)?;
    println!("Random image selected: {}", random_image_path.display());

    // 2. Composite images
    let image3_path = saved_dir.join("image3.png");
    composite_images(&random_image_path, image2_path, &image3_path)?;
    println!("Composite image saved: {}", image3_path.display());

        // 3. Create a color map between image2 and image3
    let difference_map_path = Path::new("difference_map.png");
    create_difference_map(image1_path, &image3_path, difference_map_path)?;
     println!("Difference map created: {}", difference_map_path.display());

    // 4. Run image scorer program
    let score = run_image_scorer(difference_map_path)?;
    println!("Score: {}", score);
    
    // 5. Save score to json file
    let score_path = Path::new("score.json");
     save_score_json(score, score_path)?;
    println!("Score saved to: {}", score_path.display());

    Ok(())
}

fn select_random_image(objects_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(objects_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    if entries.is_empty() {
      return Err("No files found in the objects directory".into());
    }
    let mut rng = rand::thread_rng();
    let random_image_path = entries.choose(&mut rng).ok_or("No files in objects folder")?;
    Ok(random_image_path.clone())
}

fn composite_images(
    random_image_path: &Path,
    image2_path: &Path,
    image3_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let random_img = image::open(random_image_path)?;
    let mut image2 = image::open(image2_path)?.to_rgba8();

    let (width2, height2) = image2.dimensions();
    let (width1, height1) = random_img.dimensions();

    // Generate random transformation values
    let mut rng = rand::thread_rng();
    let scale_factor: f64 = rng.gen_range(0.25..=4.0);
    let rotation_angle: i32 = rng.gen_range(0..360);
    let x_offset: u32 = rng.gen_range(0..width2);
    let y_offset: u32 = rng.gen_range(0..height2);

    // Generate random tint color
    let tint_color = Rgba([rng.gen(), rng.gen(), rng.gen(), rng.gen()]);

    let new_width = (width1 as f64 * scale_factor) as u32;
    let new_height = (height1 as f64 * scale_factor) as u32;

    // Resize the image
    let mut resized_image = DynamicImage::ImageRgba8(
        imageops::resize(&random_img, new_width, new_height, imageops::FilterType::Lanczos3),
    );

    // Apply the random tint
    apply_tint(&mut resized_image, tint_color);

    // Rotate the image
    let rotated_image = match rotation_angle {
        90 => imageops::rotate90(&resized_image.to_rgba8()),
        180 => imageops::rotate180(&resized_image.to_rgba8()),
        270 => imageops::rotate270(&resized_image.to_rgba8()),
        _ => resized_image.to_rgba8(),
    };

    let (rotated_width, rotated_height) = rotated_image.dimensions();

    // Calculate the x,y offsets to center the object
    let final_x_offset = if rotated_width < width2 {
        x_offset
    } else {
        0
    };
    let final_y_offset = if rotated_height < height2 {
        y_offset
    } else {
        0
    };

    // Composite the rotated image onto image2
    for y in 0..rotated_height {
        for x in 0..rotated_width {
            let target_x = x + final_x_offset;
            let target_y = y + final_y_offset;

            if target_x < width2 && target_y < height2 {
                let source_pixel = rotated_image.get_pixel(x, y);
                let mut target_pixel = *image2.get_pixel(target_x, target_y);

                // Alpha compositing
                let source_alpha = source_pixel[3] as f32 / 255.0;
                let target_alpha = target_pixel[3] as f32 / 255.0;

                let out_alpha = source_alpha + target_alpha * (1.0 - source_alpha);

                target_pixel[0] = ((source_pixel[0] as f32 * source_alpha + target_pixel[0] as f32 * target_alpha * (1.0 - source_alpha)) / out_alpha) as u8;
                target_pixel[1] = ((source_pixel[1] as f32 * source_alpha + target_pixel[1] as f32 * target_alpha * (1.0 - source_alpha)) / out_alpha) as u8;
                target_pixel[2] = ((source_pixel[2] as f32 * source_alpha + target_pixel[2] as f32 * target_alpha * (1.0 - source_alpha)) / out_alpha) as u8;
                target_pixel[3] = (out_alpha * 255.0).round() as u8;

                image2.put_pixel(target_x, target_y, target_pixel);
            }
        }
    }

    image2.save(image3_path)?;
    Ok(())
}


fn apply_tint(image: &mut DynamicImage, tint_color: Rgba<u8>) {
    let tint_alpha = tint_color[3] as f32 / 255.0; // Normalize tint alpha to [0, 1]
    let tint_rgb = [
        tint_color[0] as f32,
        tint_color[1] as f32,
        tint_color[2] as f32,
    ];

    image
        .as_mut_rgba8()
        .expect("Image must be RGBA")
        .pixels_mut()
        .for_each(|pixel| {
            // Extract original pixel color and alpha
            // let orig_alpha = pixel[3] as f32 / 255.0;
            let orig_rgb = [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32];

            // Blend the tint with the original color based on their alpha values
            let blended_rgb: [u8; 3] = orig_rgb
                .iter()
                .zip(tint_rgb.iter())
                .map(|(&orig, &tint)| {
                    let blended = orig * (1.0 - tint_alpha)
                        + tint * tint_alpha;
                    blended.round() as u8
                })
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap();

            // Update the pixel with the new blended color while preserving the original alpha
            pixel[0] = blended_rgb[0];
            pixel[1] = blended_rgb[1];
            pixel[2] = blended_rgb[2];
        });
}
fn create_difference_map(
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


fn run_image_scorer(input_path: &Path) -> Result<f64, Box<dyn std::error::Error>> {
    let mut command = Command::new("cargo");
    command.arg("run")
        .arg("-p")
        .arg("image_scorer")
        .arg("--")
        .arg(input_path);
    let output = command.output()?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("image_scorer program failed: {}", error_message).into());
    }

    let score_str = String::from_utf8_lossy(&output.stdout);
        let score = score_str
        .lines()
        .find(|line| line.starts_with("Image score: "))
        .and_then(|line| line.split(": ").last())
        .ok_or("Could not find score in image scorer program output")?
        .parse::<f64>()?;
    Ok(score)
}

fn save_score_json(score:f64, score_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
  let json_data = json!({
    "score": score
  });
    let mut file = fs::File::create(score_path)?;
    let json_string = serde_json::to_string_pretty(&json_data)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}