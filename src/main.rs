use std::fs::{self};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;
use imageproc::drawing::{draw_text_mut, text_size};
use image::{self, DynamicImage, ImageFormat, Rgba};
use ab_glyph::{FontArc, PxScale};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const FOLDER_PATH: &str = "./qrcodes";
const OUTPUT_DIR: &str = "./output";
const BATCH_SIZE: usize = 1000;

fn main() {
    match fs::read_dir(FOLDER_PATH) {
        Ok(entries) => {
            let bg = match image::open("images/bg.jpg") {
                Ok(img) => img,
                Err(_) => {
                    eprintln!("Error loading background image");
                    return;
                },
            };
            let start_time = Instant::now();
            let entries: Vec<_> = entries.filter_map(|entry| entry.ok()).map(|entry| entry.path()).collect();
            let total_files = entries.len();
            println!("Total files: {}", total_files);

            entries.chunks(BATCH_SIZE).for_each(|batch_files| {
                process_batch(batch_files, &bg).expect("Error processing batch");
            });

            println!("Folder processed successfully");
            let time_taken = start_time.elapsed();
            println!("Time taken: {:?}", time_taken);
        }
        Err(_) => eprintln!("Error reading folder"),
    }
}

fn process_batch(batch_files: &[PathBuf], bg: &DynamicImage) -> Result<(), Error> {
    batch_files.par_iter().for_each(|file| {
        if let Some(extension) = file.extension() {
            if extension != "DS_Store" {
                if let Err(e) = create_image(file, bg.clone()) {
                    eprintln!("Failed to create image for {}: {}", file.display(), e);
                }
            }
        }
    });
    Ok(())
}

fn create_image(file: &PathBuf, bg: DynamicImage) -> Result<(), String> {
    let qr_code_path = file.display().to_string();
    println!("{}", qr_code_path);
    let qr_code = image::open(&qr_code_path).map_err(|_| "Failed to open QR code image")?;

    let upi_id = extract_upi_id(file)?;

    println!("Stored UPI ID: {}", upi_id);

    let mut canvas = bg.to_rgba8();
    let qr_code = qr_code.resize(800, 800, image::imageops::FilterType::Lanczos3);
    image::imageops::overlay(&mut canvas, &qr_code, 150, 400);

    let font = FontArc::try_from_slice(include_bytes!("../fonts/arial.ttf")).map_err(|_| "Failed to load font")?;
    let scale = PxScale::from(50.0);
    let text_color = Rgba([0u8, 0u8, 0u8, 255u8]);

    let _text_height = text_size(scale, &font, &upi_id);
    let text_x = 240;
    let text_y = 1200;

    draw_text_mut(&mut canvas, text_color, text_x, text_y, scale, &font, &upi_id);

    let output_path = format!("{}/{}.png", OUTPUT_DIR, upi_id);
    println!("{}", output_path);
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
        }
    }

    canvas.save_with_format(&output_path, ImageFormat::Png).map_err(|e| format!("Failed to save image: {}", e))?;
    println!("Image saved successfully!");

    Ok(())
}

fn extract_upi_id(path: &PathBuf) -> Result<String, &'static str> {
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            if let Some(stripped) = file_name_str.strip_suffix(".png") {
                return Ok(stripped.to_string());
            } else {
                return Err("The file name does not have the expected extension");
            }
        } else {
            return Err("Failed to convert the file name to a string");
        }
    } else {
        return Err("Failed to extract the file name from the path");
    }
}
