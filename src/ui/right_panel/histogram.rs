use egui::{Color32, Rect, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, histogram_data: &[f32]) {
    egui::CollapsingHeader::new("📊 Histogram")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(5.0);

            let height = 100.0;
            let width = ui.available_width();
            let (container_rect, _response) =
                ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

            // 1. Draw Background Base
            ui.painter()
                .rect_filled(container_rect, 3.0, Color32::from_black_alpha(180));

            // 2. Draw Light Grid & Zone Dividers
            let stroke_soft = Stroke::new(1.0, Color32::from_white_alpha(20));
            let stroke_zone = Stroke::new(1.0, Color32::from_white_alpha(60));

            // Horizontal Mid-line
            for i in 1..=9 {
                let frac = i as f32 * 0.1;
                let y = container_rect.top() + height * frac;
                ui.painter().line_segment(
                    [
                        egui::pos2(container_rect.left(), y),
                        egui::pos2(container_rect.right(), y),
                    ],
                    stroke_soft,
                );
                let x = container_rect.left() + width * frac;
                ui.painter().line_segment(
                    [
                        egui::pos2(x, container_rect.top()),
                        egui::pos2(x, container_rect.bottom()),
                    ],
                    stroke_soft,
                );
            }

            // Vertical Zone Dividers (Blacks | Shadows | Midtones | Highlights | Whites)
            let divisions = [0.20, 0.40, 0.60, 0.80]; // 20% intervals are common in editors
            for &frac in &divisions {
                let x = container_rect.left() + width * frac;
                ui.painter().line_segment(
                    [
                        egui::pos2(x, container_rect.top()),
                        egui::pos2(x, container_rect.bottom()),
                    ],
                    stroke_zone,
                );
            }

            // 3. Draw the Histogram Bars
            if !histogram_data.is_empty() {
                let max_val = histogram_data.iter().cloned().fold(0.0, f32::max).max(1.0);
                let bar_width = width / 256.0;

                for (i, &val) in histogram_data.iter().enumerate() {
                    if val > 0.0 {
                        let normalized_height = (val / max_val) * height;

                        let x_min = container_rect.left() + (i as f32 * bar_width);
                        let x_max = x_min + bar_width;
                        let y_min = container_rect.bottom() - normalized_height;
                        let y_max = container_rect.bottom();

                        let bar_rect =
                            Rect::from_min_max(egui::pos2(x_min, y_min), egui::pos2(x_max, y_max));

                        // We use a slightly more vibrant color now that the background is gridded
                        ui.painter()
                            .rect_filled(bar_rect, 0.0, Color32::from_white_alpha(210));
                    }
                }
            }
        });
}
