use std::fs::{self, File};
use std::io::{BufWriter, Error};
use std::path::{Path, PathBuf};
use std::time::Instant;
// use imageproc::drawing::{draw_text_mut, text_size};
use image::{self, DynamicImage, GenericImageView};
// use ab_glyph::{FontArc, PxScale};
use printpdf::{BuiltinFont, ColorBits, ColorSpace, Image, ImageTransform, ImageXObject, Mm, PdfDocument, Px};
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
                if let Err(e) = save_as_pdf(file, bg.clone()) {
                    eprintln!("Failed to create image for {}: {}", file.display(), e);
                }
            }
        }
    });
    Ok(())
}

// fn create_image(file: &PathBuf, bg: DynamicImage) -> Result<(), String> {
//     let qr_code_path = file.display().to_string();
//     println!("{}", qr_code_path);
//     let qr_code = image::open(&qr_code_path).map_err(|_| "Failed to open QR code image")?;

//     let upi_id = extract_upi_id(file)?;

//     println!("Stored UPI ID: {}", upi_id);

//     let mut canvas = bg.to_rgba8();
//     let qr_code = qr_code.resize(800, 800, image::imageops::FilterType::Lanczos3);
//     image::imageops::overlay(&mut canvas, &qr_code, 150, 400);

//     let font = FontArc::try_from_slice(include_bytes!("../fonts/arial.ttf")).map_err(|_| "Failed to load font")?;
//     let scale = PxScale::from(50.0);
//     let text_color = Rgba([0u8, 0u8, 0u8, 255u8]);

//     let _text_height = text_size(scale, &font, &upi_id);
//     let text_x = 240;
//     let text_y = 1200;

//     draw_text_mut(&mut canvas, text_color, text_x, text_y, scale, &font, &upi_id);

//     let output_path = format!("{}/{}.png", OUTPUT_DIR, upi_id);
//     println!("{}", output_path);
//     if let Some(parent) = Path::new(&output_path).parent() {
//         if !parent.exists() {
//             fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
//         }
//     }
//      canvas.save_with_format(&output_path, ImageFormat::Png).map_err(|e| format!("Failed to save image: {}", e))?;
//     println!("Image saved successfully!");

//     Ok(())
// }
fn save_as_pdf(file :&PathBuf , bg : DynamicImage) -> Result<(), Box<dyn std::error::Error>>{
    let qr_code_path = file.display().to_string();
    let upi_id = extract_upi_id(file)?;
    let output_path = format!("{}/{}.pdf", OUTPUT_DIR, upi_id);
    let qr_code = match image::open(&qr_code_path).map_err(|_| "Failed to open QR code image") {
        Ok(img) => img,
        Err(_) => {
            eprintln!("Error loading background image");
            return  Err("Failed to convert the file name to a string".into());
        },
    };
    // Determine image dimensions (assuming JPG uses RGB color space)
    let (width, height) = bg.dimensions();

    // Set PDF dimensions based on image
    
    let ( doc, page1, layer1) = PdfDocument::new(&upi_id, Mm(width as f32), Mm(height as f32), "Layer 1");

    // Get the current layer
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Create ImageXObject from background image data
    let bg_image_x_object = ImageXObject {
        width: Px(width.try_into().unwrap()), // Use actual image width
        height: Px(height.try_into().unwrap()), // Use actual image height
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: bg.to_rgb8().into_raw(),
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };


    let qr_image_x_object = ImageXObject {
        width: Px(qr_code.width().try_into().unwrap()), // Use actual QR code width
        height: Px(qr_code.height().try_into().unwrap()),  // Use actual QR code height
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: qr_code.to_rgb8().into_raw(),
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    // Create Image from ImageXObject
    let qr_image = Image::from(qr_image_x_object);
    let bg_image = Image::from(bg_image_x_object);

    // Set image position using ImageTransform
    let mut bg_transform = ImageTransform::default();
    bg_transform.scale_x =  Some(11.9);
    bg_transform.scale_y = Some(11.9);
    
    let mut qr_transform = ImageTransform::default();
    qr_transform.translate_x = Some(Mm(160.0)); // Set horizontal translation to center
    qr_transform.translate_y = Some(Mm(450.0)); // Set vertical translation to center
    qr_transform.scale_x =  Some(7.0);
    qr_transform.scale_y = Some(7.0);
    // You can adjust scale_x and scale_y to resize the QR code if needed
    bg_image.add_to_layer(current_layer.clone(), bg_transform);
    qr_image.add_to_layer(current_layer.clone(), qr_transform);
     // Define the font
     let font = doc.add_builtin_font(BuiltinFont::TimesRoman).unwrap();

    let upi_id_string = match extract_upi_id(file) {
        Ok(id) => id,
        Err(_) => {
            return Err("Failed to convert the file name to a string".into());
        },
    };
    // Add the text to the current layer
    current_layer.use_text(upi_id_string, 120.0, Mm(280.0 ), Mm(380.0), &font);

    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
        }
    }

    // Save the PDF document
    doc.save(&mut BufWriter::new(File::create(&output_path).unwrap())).unwrap();

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



   