pub enum MaskType {
    Global,
    Highlights,
    Shadows,
    Whites,
    Blacks,
    // Future: Brush(Vec<f32>), RadialGradient(x, y, radius), etc.
}

pub enum EditOperation {
    Exposure(i32, MaskType),
    Contrast(i32, MaskType),
    Saturation(i32, MaskType),
    Temperature(i32, MaskType),
    Tint(i32, MaskType),
    Texture(i32, MaskType),
    Clarity(i32, MaskType),
    Dehaze(i32, MaskType),
}

// The core rendering loop.
// It remains completely ignorant of what "Exposure" or "Shadows" are.
pub fn apply_masked_edit(
    buffer: &mut [u8],
    width: u32,
    _height: u32,
    edit_fn: impl Fn(f32, f32, f32) -> (f32, f32, f32),
    mask_fn: impl Fn(u32, u32, f32, f32, f32) -> f32,
) {
    let mut idx = 0;

    for pixel in buffer.chunks_exact_mut(3) {
        let r_orig = pixel[0] as f32 / 255.0;
        let g_orig = pixel[1] as f32 / 255.0;
        let b_orig = pixel[2] as f32 / 255.0;

        let x = idx % width;
        let y = idx / width;
        idx += 1;

        let weight = mask_fn(x, y, r_orig, g_orig, b_orig);

        if weight > 0.001 {
            let (r_new, g_new, b_new) = edit_fn(r_orig, g_orig, b_orig);

            let r_final = r_orig * (1.0 - weight) + r_new * weight;
            let g_final = g_orig * (1.0 - weight) + g_new * weight;
            let b_final = b_orig * (1.0 - weight) + b_new * weight;

            pixel[0] = (r_final * 255.0).clamp(0.0, 255.0) as u8;
            pixel[1] = (g_final * 255.0).clamp(0.0, 255.0) as u8;
            pixel[2] = (b_final * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
}
