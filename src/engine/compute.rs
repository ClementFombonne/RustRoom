use image::DynamicImage;

use super::{
    basic,
    pipeline::{self, EditOperation, MaskType},
};
use crate::ui::right_panel::PhotoAdjustments;

pub fn apply_all_adjustments(img: &DynamicImage, adj: &PhotoAdjustments) -> egui::ColorImage {
    let mut rgb_img = img.to_rgb8();
    let width = rgb_img.width();
    let height = rgb_img.height();

    let mut binding = rgb_img.as_flat_samples_mut();
    let buffer = binding.as_mut_slice();

    // 1. BUILD THE EDIT STACK
    // This is incredibly powerful. If you want to change the order of operations,
    // you simply change the order you push them into this vector!
    let mut stack = Vec::new();

    if adj.exposure != 50 {
        stack.push(EditOperation::Exposure(adj.exposure, MaskType::Global));
    }
    if adj.highlights != 50 {
        stack.push(EditOperation::Exposure(
            adj.highlights,
            MaskType::Highlights,
        ));
    }
    if adj.shadows != 50 {
        stack.push(EditOperation::Exposure(adj.shadows, MaskType::Shadows));
    }
    if adj.whites != 50 {
        stack.push(EditOperation::Exposure(adj.whites, MaskType::Whites));
    }
    if adj.blacks != 50 {
        stack.push(EditOperation::Exposure(adj.blacks, MaskType::Blacks));
    }
    if adj.contrast != 50 {
        stack.push(EditOperation::Contrast(adj.contrast, MaskType::Global));
    }
    if adj.saturation != 50 {
        stack.push(EditOperation::Saturation(adj.saturation, MaskType::Global));
    }
    if adj.temperature != 50 {
        stack.push(EditOperation::Temperature(
            adj.temperature,
            MaskType::Global,
        ));
    }
    if adj.tint != 50 {
        stack.push(EditOperation::Tint(adj.tint, MaskType::Global));
    }
    if adj.texture != 50 {
        stack.push(EditOperation::Texture(adj.texture, MaskType::Global));
    }
    if adj.clarity != 50 {
        stack.push(EditOperation::Clarity(adj.clarity, MaskType::Global));
    }
    if adj.dehaze != 50 {
        stack.push(EditOperation::Dehaze(adj.dehaze, MaskType::Global));
    }

    // 2. EXECUTE THE STACK
    for op in stack {
        match op {
            EditOperation::Exposure(val, mask_type) => {
                let edit_fn = basic::create_exposure_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    MaskType::Highlights => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::highlight_mask(),
                    ),
                    MaskType::Shadows => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::shadow_mask(),
                    ),
                    MaskType::Whites => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::whites_mask(),
                    ),
                    MaskType::Blacks => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::blacks_mask(),
                    ),
                }
            }
            EditOperation::Contrast(val, mask_type) => {
                let edit_fn = basic::create_contrast_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {} // We could easily add targeted contrast later!
                }
            }
            EditOperation::Saturation(val, mask_type) => {
                let edit_fn = basic::create_saturation_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
            EditOperation::Temperature(val, mask_type) => {
                let edit_fn = basic::create_temperature_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
            EditOperation::Tint(val, mask_type) => {
                let edit_fn = basic::create_tint_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
            EditOperation::Texture(val, mask_type) => {
                let edit_fn = basic::create_texture_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
            EditOperation::Clarity(val, mask_type) => {
                let edit_fn = basic::create_clarity_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
            EditOperation::Dehaze(val, mask_type) => {
                let edit_fn = basic::create_dehaze_edit(val);
                match mask_type {
                    MaskType::Global => pipeline::apply_masked_edit(
                        buffer,
                        width,
                        height,
                        edit_fn,
                        basic::global_mask(),
                    ),
                    _ => {}
                }
            }
        }
    }

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

pub fn calculate_histogram_from_buffer(rgb_samples: &[u8]) -> Vec<f32> {
    let mut bins = vec![0.0; 256];

    // Process in chunks of 3 (R, G, B)
    for pixel in rgb_samples.chunks_exact(3) {
        // Standard Luminance calculation
        let lum = (0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32)
            as usize;
        bins[lum.clamp(0, 255)] += 1.0;
    }

    // Normalize
    let max_val = bins.iter().cloned().fold(0.0, f32::max);
    if max_val > 0.0 {
        for val in bins.iter_mut() {
            *val /= max_val;
        }
    }

    bins
}

pub fn load_image(file_path: &str) -> Option<DynamicImage> {
    let clean_path = if file_path.starts_with("file://") {
        &file_path[7..]
    } else {
        file_path
    };
    image::open(clean_path).ok()
}
