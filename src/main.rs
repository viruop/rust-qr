

use std::fs::{self};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;
use imageproc::drawing::{draw_text_mut, text_size};

use image::ImageFormat;
use ab_glyph::{FontArc, PxScale};


const FOLDER_PATH: &str = "./qrcodes";
const OUTPUT_DIR: &str = "./output";
const BATCH_SIZE: usize = 1000;

fn main() {
    if let Ok(entries) = fs::read_dir(FOLDER_PATH) {
        let bg  = match image::open("images/bg.jpg") {
            Ok(img) => img,
            Err(_) => return (),
        };
        let start_time = Instant::now();
        let entries: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect();
        let total_files = entries.len();
        println!("{}", total_files);
        let mut processed_files = 0;

        while processed_files < total_files {
            let batch_files = &entries[processed_files..(processed_files + BATCH_SIZE).min(total_files)];
            process_batch(batch_files , bg.clone()).expect("Error processing batch");
            processed_files += BATCH_SIZE;
        }

        println!("Folder created successfully");
        let time_taken = start_time.elapsed();
        println!("Time taken: {:?}", time_taken);
    } else {
        eprintln!("Error reading folder");
    }
}

fn process_batch(batch_files: &[std::path::PathBuf] , bg :image::DynamicImage) -> Result<(), Error> {
    for file in batch_files {
        if let Some(extension) = file.extension() {
            if extension != "DS_Store" {
                create_image(file , bg.clone());
            }
        }
    }
    Ok(())
}


fn create_image(file: &PathBuf , bg :image::DynamicImage ) -> bool {
    let qr_code_path = format!("{}", file.display());
    println!("{}",file.display());
    let qr_code = match image::open(qr_code_path) {
        Ok(img) => img,
        Err(_) => return false,
    };
        // Extract and store the result in a variable
        let upi_id_result = extract_upi_id(&file);
            
        // Handle the result
        let upi_id = match upi_id_result {
            Ok(upi_id) => {
                upi_id // Store the extracted UPI ID in the variable
            },
            Err(e) => {
                println!("Error: {}", e);
                String::new() // Return an empty string or handle the error as needed
            },
        };

    // The UPI ID is now stored in the upi_id variable
    println!("Stored UPI ID: {}", upi_id);
   // Create canvas with the same dimensions as the background image
    let mut canvas = bg.to_rgba8();
    let qr_code = qr_code.resize(800, 800, image::imageops::FilterType::Lanczos3);
    let qr_x = 150;
    let qr_y = 400;
    image::imageops::overlay(&mut canvas, &qr_code, qr_x, qr_y);
   

   // Add UPI ID text to the image
//    let font_data = (include_bytes!("../fonts/arial.ttf")); // Path to the font file
   let font = FontArc::try_from_slice((include_bytes!("../fonts/arial.ttf"))).unwrap();
   let scale = PxScale::from(50.0); // Adjust scale as needed

   let text_color = image::Rgba([0u8, 0u8, 0u8, 255u8]); // Black text

   // Calculate the position to draw the text (centered)
   let (text_width, text_height) = text_size(scale, &font, &upi_id);

   draw_text_mut(
    &mut canvas, 
    text_color, 240, 1200,
    scale, 
    &font, 
    &upi_id);
    let output_path = format!("{}/{}.png", OUTPUT_DIR, upi_id);
    println!("{}",output_path);
     // Create the directory if it doesn't exist
     if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directories: {}", e);
                return false
            }
        }
    }
    match canvas.save_with_format(output_path, ImageFormat::Png) {
        Ok(_) => println!("Image saved successfully!"),
        Err(e) => eprintln!("Failed to save image: {}", e),
    };

    return true
}



fn extract_upi_id(path: &PathBuf) -> Result<String, &'static str> {
    
    // Extract the file name from the path
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            // Strip the extension to get the part we need
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

