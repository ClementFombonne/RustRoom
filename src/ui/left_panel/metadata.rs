use egui::{RichText, TextWrapMode};
use serde_json::Value;

pub fn show(ui: &mut egui::Ui, metadata_json: Option<&str>) {
    egui::CollapsingHeader::new("ℹ Image Metadata")
        .default_open(true)
        .show(ui, |ui| {
            if let Some(json_str) = metadata_json {
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    let col_width = (ui.available_width() * 0.4).max(80.0);

                    egui::Grid::new("metadata_grid")
                        .num_columns(2)
                        .striped(true)
                        .min_col_width(col_width)
                        .show(ui, |ui| {
                            if let Some(obj) = parsed.as_object() {
                                for (key, val) in obj {
                                    let mut cap_key = key.clone();
                                    if let Some(r) = cap_key.get_mut(0..1) {
                                        r.make_ascii_uppercase();
                                    }

                                    let key_text = format!("{}:", cap_key);

                                    ui.add(
                                        egui::Label::new(RichText::new(&key_text).strong())
                                            .wrap_mode(TextWrapMode::Truncate),
                                    )
                                    .on_hover_text(&key_text);

                                    let text = if let Some(s) = val.as_str() {
                                        s.to_string()
                                    } else {
                                        val.to_string()
                                    };

                                    ui.add(egui::Label::new(text).wrap_mode(TextWrapMode::Wrap));
                                    ui.end_row();
                                }
                            }
                        });
                } else {
                    ui.label("Failed to parse metadata.");
                }
            } else {
                ui.label("No image selected.");
            }
        });
}
