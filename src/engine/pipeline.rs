// The generic pipeline applier.
// E: The edit function mapping (R, G, B) to (R, G, B)
// M: The mask function mapping (X, Y, R, G, B) to a weight between 0.0 and 1.0
pub fn apply_masked_edit(
    buffer: &mut [u8],
    width: u32,
    _height: u32,
    edit_fn: impl Fn(f32, f32, f32) -> (f32, f32, f32),
    mask_fn: impl Fn(u32, u32, f32, f32, f32) -> f32,
) {
    let mut idx = 0;

    // Iterate over every pixel in the raw image buffer
    for pixel in buffer.chunks_exact_mut(3) {
        let r_orig = pixel[0] as f32 / 255.0;
        let g_orig = pixel[1] as f32 / 255.0;
        let b_orig = pixel[2] as f32 / 255.0;

        let x = idx % width;
        let y = idx / width;
        idx += 1;

        // 1. Evaluate the mask for this specific pixel
        let weight = mask_fn(x, y, r_orig, g_orig, b_orig);

        // Optimization: Don't process pixels where the mask is 0
        if weight > 0.001 {
            // 2. Evaluate the edit
            let (r_new, g_new, b_new) = edit_fn(r_orig, g_orig, b_orig);

            // 3. Blend based on the mask weight (Linear Interpolation)
            let r_final = r_orig * (1.0 - weight) + r_new * weight;
            let g_final = g_orig * (1.0 - weight) + g_new * weight;
            let b_final = b_orig * (1.0 - weight) + b_new * weight;

            // 4. Write back to the buffer
            pixel[0] = (r_final * 255.0).clamp(0.0, 255.0) as u8;
            pixel[1] = (g_final * 255.0).clamp(0.0, 255.0) as u8;
            pixel[2] = (b_final * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
}
