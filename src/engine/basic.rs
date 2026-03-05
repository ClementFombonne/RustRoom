// --- THE EDITS ---

/// Returns a closure that applies an Exposure shift (EV)
pub fn create_exposure_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    // Convert 0-100 slider into -2.0 to +2.0 Exposure Values (Stops)
    let ev = (slider_val as f32 - 50.0) / 25.0;
    let multiplier = 2.0_f32.powf(ev);

    move |r, g, b| (r * multiplier, g * multiplier, b * multiplier)
}

pub fn create_contrast_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    // A slight curve (powf) makes the slider feel more natural and less aggressive
    let factor = (slider_val as f32 / 50.0).powf(1.2);

    move |r, g, b| {
        (
            (r - 0.5) * factor + 0.5,
            (g - 0.5) * factor + 0.5,
            (b - 0.5) * factor + 0.5,
        )
    }
}

/// Returns a closure that applies Saturation
pub fn create_saturation_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let factor = slider_val as f32 / 50.0;

    move |r, g, b| {
        // Find the absolute grayscale value of the pixel
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        (
            lum + (r - lum) * factor,
            lum + (g - lum) * factor,
            lum + (b - lum) * factor,
        )
    }
}

pub fn create_temperature_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let temp_shift = (slider_val as f32 - 50.0) / 250.0;
    move |r, g, b| {
        let r = r + temp_shift;
        let g = g + (temp_shift * 0.5);
        let b = b - temp_shift;
        (r, g, b)
    }
}

pub fn create_tint_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let tint_shift = (slider_val as f32 - 50.0) / 250.0;
    move |r, g, b| {
        let r = r + tint_shift;
        let g = g - (tint_shift * 1.0);
        let b = b + tint_shift;

        (r, g, b)
    }
}

pub fn create_texture_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let intensity = (slider_val as f32 - 50.0) / 100.0;

    move |r, g, b| {
        // A simple approximation of texture enhancement using an S-Curve
        // focused on the luminance differences.
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let factor = 1.0 + intensity * (1.0 - (2.0 * l - 1.0).abs());

        (r * factor, g * factor, b * factor)
    }
}

/// Clarity: Mid-tone contrast adjustment
pub fn create_clarity_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let intensity = (slider_val as f32 - 50.0) / 50.0;

    move |r, g, b| {
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        // Target the mid-tones (around 0.5)
        let midtone_mask = 1.0 - (2.0 * l - 1.0).abs();
        let factor = 1.0 + intensity * midtone_mask * (l - 0.5);

        (r * factor, g * factor, b * factor)
    }
}

/// Dehaze: Combined contrast, saturation, and black-level shift
pub fn create_dehaze_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    let intensity = (slider_val as f32 - 50.0) / 100.0;

    move |r, g, b| {
        // 1. Calculate luminance
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        // 2. Black point shift: We pull the floor down based on intensity
        // This is more aggressive in the shadows (where haze is most visible)
        let black_shift = intensity * (1.0 - l).powf(2.0);

        let mut nr = r - black_shift;
        let mut ng = g - black_shift;
        let mut nb = b - black_shift;

        // 3. Contrast boost: Expand the remaining range
        let contrast_factor = 1.0 + intensity * 0.5;
        nr = (nr - 0.5) * contrast_factor + 0.5;
        ng = (ng - 0.5) * contrast_factor + 0.5;
        nb = (nb - 0.5) * contrast_factor + 0.5;

        // 4. Saturation boost: Haze kills color, so we bring it back
        let nl = 0.2126 * nr + 0.7152 * ng + 0.0722 * nb;
        let sat_factor = 1.0 + intensity * 0.8;

        (
            nl + (nr - nl) * sat_factor,
            nl + (ng - nl) * sat_factor,
            nl + (nb - nl) * sat_factor,
        )
    }
}

// --- THE MASKS ---

/// A universal mask that applies the edit at 100% everywhere
pub fn global_mask() -> impl Fn(u32, u32, f32, f32, f32) -> f32 {
    |_x, _y, _r, _g, _b| 1.0
}

/// A mask targeting only the brightest pixels (Highlights)
pub fn highlight_mask() -> impl Fn(u32, u32, f32, f32, f32) -> f32 {
    |_x, _y, r, g, b| {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        // Ramps from 0.0 at mid-gray to 1.0 at pure white
        if lum < 0.5 { 0.0 } else { (lum - 0.5) * 2.0 }
    }
}

/// A mask targeting only the darkest pixels (Shadows)
pub fn shadow_mask() -> impl Fn(u32, u32, f32, f32, f32) -> f32 {
    |_x, _y, r, g, b| {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        // Ramps from 1.0 at pure black to 0.0 at mid-gray
        if lum > 0.5 { 0.0 } else { 1.0 - (lum * 2.0) }
    }
}

/// A mask targeting ONLY extreme whites
pub fn whites_mask() -> impl Fn(u32, u32, f32, f32, f32) -> f32 {
    |_x, _y, r, g, b| {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if lum < 0.75 { 0.0 } else { (lum - 0.75) * 4.0 }
    }
}

/// A mask targeting ONLY extreme blacks
pub fn blacks_mask() -> impl Fn(u32, u32, f32, f32, f32) -> f32 {
    |_x, _y, r, g, b| {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if lum > 0.25 { 0.0 } else { 1.0 - (lum * 4.0) }
    }
}
