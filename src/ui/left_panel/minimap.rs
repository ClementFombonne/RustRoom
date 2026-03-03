use crate::ui::scene::ImageViewer;
use egui::{Color32, Rect, Stroke, StrokeKind, Vec2, emath::TSTransform};

pub fn show(ui: &mut egui::Ui, viewer: &mut ImageViewer, image_path: &str) {
    egui::CollapsingHeader::new("🗺 Navigation & Zoom")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(5.0);

            // A. Zoom Buttons
            ui.horizontal(|ui| {
                if ui.button("Fit").clicked() && viewer.viewport_rect.is_positive() {
                    let scale_x = viewer.viewport_rect.width() / viewer.world_size.x;
                    let scale_y = viewer.viewport_rect.height() / viewer.world_size.y;
                    viewer.transform.scaling = scale_x.min(scale_y);
                    viewer.transform.translation = viewer.viewport_rect.center().to_vec2();
                }
                if ui.button("Fill").clicked() && viewer.viewport_rect.is_positive() {
                    let scale_x = viewer.viewport_rect.width() / viewer.world_size.x;
                    let scale_y = viewer.viewport_rect.height() / viewer.world_size.y;
                    viewer.transform.scaling = scale_x.max(scale_y);
                    viewer.transform.translation = viewer.viewport_rect.center().to_vec2();
                }
                if ui.button("1:1").clicked() && viewer.viewport_rect.is_positive() {
                    viewer.transform.scaling = 1.0;
                    viewer.transform.translation = viewer.viewport_rect.center().to_vec2();
                }
            });

            ui.add_space(5.0);

            // B. Minimap
            let minimap_width = ui.available_width();
            let square_size = Vec2::splat(minimap_width);
            let (_id, square_rect) = ui.allocate_space(square_size);

            ui.painter()
                .rect_filled(square_rect, 5.0, Color32::from_black_alpha(150));
            ui.painter().rect_stroke(
                square_rect,
                5.0,
                Stroke::new(1.0, Color32::from_gray(100)),
                StrokeKind::Outside,
            );

            let aspect = viewer.world_size.x / viewer.world_size.y;
            let mut image_size = square_size;
            if aspect > 1.0 {
                image_size.y = minimap_width / aspect;
            } else {
                image_size.x = minimap_width * aspect;
            }

            let image_rect = Rect::from_center_size(square_rect.center(), image_size);

            ui.put(
                image_rect,
                egui::Image::new(image_path).fit_to_exact_size(image_size),
            );

            let minimap_scale = image_size.x / viewer.world_size.x;
            let world_to_minimap = TSTransform::from_translation(image_rect.center().to_vec2())
                * TSTransform::from_scaling(minimap_scale);

            if viewer.viewport_rect.is_positive() {
                let from_screen = viewer.transform.inverse();
                let screen_in_scene = Rect::from_min_max(
                    from_screen * viewer.viewport_rect.min,
                    from_screen * viewer.viewport_rect.max,
                );

                let indicator_rect = world_to_minimap.mul_rect(screen_in_scene);

                ui.painter().rect_stroke(
                    indicator_rect.intersect(square_rect),
                    0.0,
                    Stroke::new(2.0, Color32::YELLOW),
                    StrokeKind::Inside,
                );
            }

            // let response = ui.interact(square_rect, id, egui::Sense::click_and_drag());
            let response = ui.allocate_rect(square_rect, egui::Sense::click_and_drag());

            if response.dragged() {
                let delta_m = response.drag_delta();
                let delta_w = delta_m / minimap_scale;
                viewer.transform.translation -= delta_w * viewer.transform.scaling;
            } else if response.clicked() {
                if let Some(pos_m) = response.interact_pointer_pos() {
                    let minimap_to_world = world_to_minimap.inverse();
                    let screen_center_w = minimap_to_world * pos_m;

                    viewer.transform.translation = viewer.viewport_rect.center().to_vec2()
                        - viewer.transform.scaling * screen_center_w.to_vec2();
                }
            }

            ui.add_space(5.0);

            // C. Zoom Slider
            let mut zoom_pct = viewer.transform.scaling * 100.0;
            ui.horizontal(|ui| {
                ui.label("Zoom %:");
                if ui
                    .add(egui::Slider::new(&mut zoom_pct, 10.0..=500.0))
                    .changed()
                {
                    viewer.transform.scaling = zoom_pct / 100.0;
                }
            });
        });
}
