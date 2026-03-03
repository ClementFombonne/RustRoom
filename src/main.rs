use eframe::*;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/RustRoom_icon.png");
    let icon =
        eframe::icon_data::from_png_bytes(icon_bytes).expect("Failed to load application icon");

    let app_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(Arc::new(icon))
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    run_native(
        "RustRoom",
        app_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(rustroom::ui::RustRoomApp::default()))
        }),
    )
}
