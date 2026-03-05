use crate::ui::app::RustRoomApp;
use egui::Context;

pub fn show(ctx: &Context, app: &RustRoomApp) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(format!(
                    "📏 Dimensions: {:.0} x {:.0} px",
                    app.viewer.world_size.x, app.viewer.world_size.y
                ));
                ui.separator();
                ui.label(format!(
                    "🔍 Zoom: {:.0}%",
                    app.viewer.transform.scaling * 100.0
                ));
                ui.separator();
                ui.label(format!("📁 File: {}", app.current_image));

                if app.histogram_rx.is_some() {
                    ui.separator();
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label("Computing Histogram...");
                }

                if app.import_rx.is_some() {
                    ui.separator();
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label("Importing...");
                }

                if app.is_computing_edit {
                    ui.separator();
                    ui.add(egui::Spinner::new().size(12.0));
                    ui.label("Rendering Edit...");
                }
            });
        });
}
