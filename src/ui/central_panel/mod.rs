pub mod scene;

use self::scene::ImageViewer;
use egui::Context;

pub fn show(
    ctx: &Context,
    viewer: &mut ImageViewer,
    image_path: &str,
    texture: &Option<egui::TextureHandle>,
    left_visible: &mut bool,
    right_visible: &mut bool,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        viewer.show(ui, image_path, texture, left_visible, right_visible);
    });
}
