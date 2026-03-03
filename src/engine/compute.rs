use image::DynamicImage;

use super::{basic, pipeline};
use crate::ui::right_panel::PhotoAdjustments;

pub fn apply_all_adjustments(img: &DynamicImage, adj: &PhotoAdjustments) -> egui::ColorImage {
    let mut rgb_img = img.to_rgb8();
    let width = rgb_img.width();
    let height = rgb_img.height();

    // We get direct, mutable access to the raw pixel bytes
    let mut binding = rgb_img.as_flat_samples_mut();
    let buffer = binding.as_mut_slice();

    // 1. General Exposure (Applies everywhere)
    if adj.exposure != 50 {
        pipeline::apply_masked_edit(
            buffer,
            width,
            height,
            basic::create_exposure_edit(adj.exposure),
            basic::global_mask(),
        );
    }

    // 2. Highlights (Exposure shift applied ONLY to brights)
    if adj.highlights != 50 {
        pipeline::apply_masked_edit(
            buffer,
            width,
            height,
            basic::create_exposure_edit(adj.highlights),
            basic::highlight_mask(),
        );
    }

    // 3. Shadows (Exposure shift applied ONLY to darks)
    if adj.shadows != 50 {
        pipeline::apply_masked_edit(
            buffer,
            width,
            height,
            basic::create_exposure_edit(adj.shadows),
            basic::shadow_mask(),
        );
    }

    // 4. Whites (Exposure shift applied ONLY to extreme brights)
    if adj.whites != 50 {
        pipeline::apply_masked_edit(
            buffer,
            width,
            height,
            basic::create_exposure_edit(adj.whites),
            basic::whites_mask(),
        );
    }

    // 5. Blacks (Exposure shift applied ONLY to extreme darks)
    if adj.blacks != 50 {
        pipeline::apply_masked_edit(
            buffer,
            width,
            height,
            basic::create_exposure_edit(adj.blacks),
            basic::blacks_mask(),
        );
    }

    // Convert back to egui format
    let size = [width as _, height as _];
    egui::ColorImage::from_rgb(size, rgb_img.as_raw())
}

pub fn calculate_histogram(file_path: &str) -> Option<Vec<f32>> {
    // 1. Clean the path perfectly
    let path = if file_path.starts_with("file://") {
        &file_path[7..]
    } else if file_path.starts_with("http") {
        // Skip web images
        return None;
    } else {
        // It's a normal path, just use it as-is!
        file_path
    };

    // 2. Open the image using the `image` crate
    let img = image::open(path).ok()?;

    // 3. Convert to 8-bit grayscale
    let luma = img.to_luma8();

    // 4. Count the pixels
    let mut bins = vec![0.0; 256];
    for pixel in luma.pixels() {
        let brightness = pixel[0] as usize;
        bins[brightness] += 1.0;
    }

    // 5. Normalize the data between 0.0 and 1.0
    let max_val = bins.iter().cloned().fold(0.0, f32::max);
    if max_val > 0.0 {
        for val in bins.iter_mut() {
            *val /= max_val;
        }
    }

    Some(bins)
}

pub fn load_image(file_path: &str) -> Option<DynamicImage> {
    let clean_path = if file_path.starts_with("file://") {
        &file_path[7..]
    } else {
        file_path
    };
    image::open(clean_path).ok()
}
