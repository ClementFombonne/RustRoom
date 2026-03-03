pub mod basic_edit;
pub mod histogram;

#[derive(Clone)]
pub struct PhotoAdjustments {
    // White Balance
    pub temperature: i32, // 0 to 100 (50 is neutral)
    pub tint: i32,        // 0 to 100 (50 is neutral)
    // Light
    pub exposure: i32,
    pub contrast: i32,
    pub highlights: i32,
    pub shadows: i32,
    pub whites: i32,
    pub blacks: i32,
    // Detail
    pub texture: i32,
    pub clarity: i32,
    pub dehaze: i32,
    // Color
    pub hue: i32,
    pub saturation: i32,
}

impl Default for PhotoAdjustments {
    fn default() -> Self {
        Self {
            temperature: 50,
            tint: 50,
            exposure: 50,
            contrast: 50,
            highlights: 50,
            shadows: 50,
            whites: 50,
            blacks: 50,
            texture: 50,
            clarity: 50,
            dehaze: 50,
            hue: 50,
            saturation: 50,
        }
    }
}

pub struct RightPanel {
    pub adjustments: PhotoAdjustments,
    pub histogram_data: Vec<f32>,
}

impl Default for RightPanel {
    fn default() -> Self {
        Self {
            adjustments: PhotoAdjustments::default(),
            histogram_data: vec![0.0; 256],
        }
    }
}

impl RightPanel {
    pub fn show(&mut self, ctx: &egui::Context, is_visible: bool) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .show_animated(ctx, is_visible, |ui| {
                // Wrap the entire panel in a Vertical Scroll Area
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(5.0);

                        // 1. Render the Histogram Submodule
                        histogram::show(ui, &self.histogram_data);

                        ui.add_space(10.0);

                        // 2. Render the Basic Edits Submodule
                        basic_edit::show(ui, &mut self.adjustments);

                        ui.add_space(10.0);
                    });
            });
    }
}
