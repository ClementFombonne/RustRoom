// --- THE EDITS ---

/// Returns a closure that applies an Exposure shift (EV)
pub fn create_exposure_edit(slider_val: i32) -> impl Fn(f32, f32, f32) -> (f32, f32, f32) {
    // Convert 0-100 slider into -2.0 to +2.0 Exposure Values (Stops)
    let ev = (slider_val as f32 - 50.0) / 25.0;
    let multiplier = 2.0_f32.powf(ev);

    move |r, g, b| (r * multiplier, g * multiplier, b * multiplier)
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
