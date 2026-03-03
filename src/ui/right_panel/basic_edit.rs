use egui::{Color32, Mesh, Rect, Shape, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, adj: &mut super::PhotoAdjustments) {
    let defaults = super::PhotoAdjustments::default();

    egui::CollapsingHeader::new("Basic Edits")
        .default_open(true)
        .show(ui, |ui| {
            // --- Group: White Balance ---
            ui.label(egui::RichText::new("White Balance").small().weak());
            draw_gradient_slider(
                ui,
                "Temp",
                &mut adj.temperature,
                defaults.temperature,
                Color32::from_rgb(50, 50, 255),
                Color32::from_rgb(255, 255, 50),
            );
            draw_gradient_slider(
                ui,
                "Tint",
                &mut adj.tint,
                defaults.tint,
                Color32::from_rgb(50, 255, 50),
                Color32::from_rgb(255, 50, 255),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // --- Group: Light ---
            ui.label(egui::RichText::new("Light").small().weak());
            render_slider(ui, "Exposure", &mut adj.exposure, defaults.exposure);
            render_slider(ui, "Contrast", &mut adj.contrast, defaults.contrast);
            render_slider(ui, "Highlights", &mut adj.highlights, defaults.highlights);
            render_slider(ui, "Shadows", &mut adj.shadows, defaults.shadows);
            render_slider(ui, "Whites", &mut adj.whites, defaults.whites);
            render_slider(ui, "Blacks", &mut adj.blacks, defaults.blacks);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // --- Group: Detail ---
            ui.label(egui::RichText::new("Detail").small().weak());
            render_slider(ui, "Texture", &mut adj.texture, defaults.texture);
            render_slider(ui, "Clarity", &mut adj.clarity, defaults.clarity);
            render_slider(ui, "Dehaze", &mut adj.dehaze, defaults.dehaze);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // --- Group: Color ---
            ui.label(egui::RichText::new("Color").small().weak());
            draw_hue_slider(ui, "Hue", &mut adj.hue, defaults.hue);
            draw_gradient_slider(
                ui,
                "Saturation",
                &mut adj.saturation,
                defaults.saturation,
                Color32::from_gray(100),
                Color32::from_rgb(255, 50, 50),
            );
        });
}

fn draw_reset_label(ui: &mut egui::Ui, label: &str, value: &mut i32, default_val: i32) {
    let label_resp = ui
        .allocate_ui_with_layout(
            egui::vec2(70.0, 20.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let resp = ui.add(egui::Label::new(label).sense(egui::Sense::click()));

                if resp.hovered() {
                    let y = resp.rect.bottom();
                    ui.painter().line_segment(
                        [
                            egui::pos2(resp.rect.left(), y),
                            egui::pos2(resp.rect.right(), y),
                        ],
                        egui::Stroke::new(1.0, ui.visuals().text_color()),
                    );
                }
                resp
            },
        )
        .inner;

    if label_resp.clicked() || label_resp.double_clicked() {
        *value = default_val;
    }

    label_resp
        .on_hover_text("Click to reset")
        .on_hover_cursor(egui::CursorIcon::PointingHand);
}

// --- STANDARD SLIDER ---
fn render_slider(ui: &mut egui::Ui, label: &str, value: &mut i32, default_val: i32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        draw_reset_label(ui, label, value, default_val);
        ui.add(egui::Slider::new(value, 0..=100));
    });
}

// --- GRADIENT SLIDER ---
fn draw_gradient_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut i32,
    default_val: i32,
    left: Color32,
    right: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        draw_reset_label(ui, label, value, default_val);

        let slider_width = ui.spacing().slider_width;
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(slider_width, 20.0), egui::Sense::hover());

        let track_height = ui.style().spacing.slider_rail_height;
        let v_rect = Rect::from_center_size(rect.center(), Vec2::new(slider_width, track_height));

        let radius = 2.5;

        add_rounded_gradient_mesh(ui, v_rect, radius, |x| {
            let t = ((x - v_rect.left()) / v_rect.width()).clamp(0.0, 1.0);
            Color32::from_rgb(
                (left.r() as f32 * (1.0 - t) + right.r() as f32 * t) as u8,
                (left.g() as f32 * (1.0 - t) + right.g() as f32 * t) as u8,
                (left.b() as f32 * (1.0 - t) + right.b() as f32 * t) as u8,
            )
        });

        let old_visuals = ui.visuals().clone();
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
        ui.visuals_mut().extreme_bg_color = Color32::TRANSPARENT;

        ui.put(rect, egui::Slider::new(value, 0..=100));

        *ui.visuals_mut() = old_visuals;
    });
}

// --- HUE SLIDER ---
fn draw_hue_slider(ui: &mut egui::Ui, label: &str, value: &mut i32, default_val: i32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        draw_reset_label(ui, label, value, default_val);

        let slider_width = ui.spacing().slider_width;
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(slider_width, 20.0), egui::Sense::hover());

        let track_height = ui.style().spacing.slider_rail_height;
        let v_rect = Rect::from_center_size(rect.center(), Vec2::new(slider_width, track_height));

        let radius = 2.5;

        add_rounded_gradient_mesh(ui, v_rect, radius, |x| {
            let t = ((x - v_rect.left()) / v_rect.width()).clamp(0.0, 1.0);
            egui::epaint::Hsva::new(t, 1.0, 1.0, 1.0).into()
        });

        let old_visuals = ui.visuals().clone();
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
        ui.visuals_mut().extreme_bg_color = Color32::TRANSPARENT;

        ui.put(rect, egui::Slider::new(value, 0..=100));

        *ui.visuals_mut() = old_visuals;
    });
}

// --- THE MESH MATH HELPER ---
fn add_rounded_gradient_mesh(
    ui: &mut egui::Ui,
    rect: Rect,
    radius: f32,
    color_at_x: impl Fn(f32) -> Color32,
) {
    let uv = egui::epaint::WHITE_UV;
    let mut mesh = Mesh::default();
    let r = radius.clamp(0.0, (rect.width() / 2.0).min(rect.height() / 2.0));

    let left_center = rect.left() + r;
    let right_center = rect.right() - r;

    let mut xs = Vec::new();

    let corner_steps = 4;
    for i in 0..=corner_steps {
        let angle =
            std::f32::consts::PI + std::f32::consts::FRAC_PI_2 * (i as f32 / corner_steps as f32);
        xs.push(left_center + r * angle.cos());
    }

    let mid_steps = 15;
    for i in 1..mid_steps {
        let t = i as f32 / mid_steps as f32;
        xs.push(left_center + t * (right_center - left_center));
    }

    for i in 0..=corner_steps {
        let angle = -std::f32::consts::FRAC_PI_2
            + std::f32::consts::FRAC_PI_2 * (i as f32 / corner_steps as f32);
        xs.push(right_center + r * angle.cos());
    }

    for window in xs.windows(2) {
        let x0 = window[0];
        let x1 = window[1];
        if x1 - x0 < 0.05 {
            continue;
        }

        let get_y = |x: f32| -> (f32, f32) {
            if x < left_center {
                let dx = left_center - x;
                let dy = (r * r - dx * dx).max(0.0).sqrt();
                (rect.top() + r - dy, rect.bottom() - r + dy)
            } else if x > right_center {
                let dx = x - right_center;
                let dy = (r * r - dx * dx).max(0.0).sqrt();
                (rect.top() + r - dy, rect.bottom() - r + dy)
            } else {
                (rect.top(), rect.bottom())
            }
        };

        let (y0_top, y0_bottom) = get_y(x0);
        let (y1_top, y1_bottom) = get_y(x1);

        let c0 = color_at_x(x0);
        let c1 = color_at_x(x1);

        let idx = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: egui::pos2(x0, y0_top),
            uv,
            color: c0,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: egui::pos2(x1, y1_top),
            uv,
            color: c1,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: egui::pos2(x1, y1_bottom),
            uv,
            color: c1,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: egui::pos2(x0, y0_bottom),
            uv,
            color: c0,
        });

        mesh.indices
            .extend([idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
    }

    ui.painter().add(Shape::mesh(mesh));
}
