pub mod metadata;
pub mod minimap;

use crate::ui::scene::ImageViewer;

pub fn show(
    ctx: &egui::Context,
    is_visible: bool,
    viewer: &mut ImageViewer,
    image_path: &str,
    metadata_json: Option<&str>,
) {
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(220.0)
        .show_animated(ctx, is_visible, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(5.0);

                    // 1. Render Minimap Submodule
                    minimap::show(ui, viewer, image_path);

                    ui.add_space(10.0);

                    // 2. Render Metadata Submodule
                    metadata::show(ui, metadata_json);

                    ui.add_space(10.0);
                });
        });
}
