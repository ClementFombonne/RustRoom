use egui::{Color32, Pos2, Rect, Vec2, emath::TSTransform};

pub struct ImageViewer {
    pub transform: TSTransform,
    pub world_size: Vec2,
    pub viewport_rect: Rect,
    pub recenter_requested: bool,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            transform: TSTransform::default(),
            world_size: Vec2::new(800.0, 600.0),
            viewport_rect: Rect::NOTHING,
            recenter_requested: true,
        }
    }
}

impl ImageViewer {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        image_path: &str,
        dynamic_texture: &Option<egui::TextureHandle>,
        left_panel_visible: &mut bool,
        right_panel_visible: &mut bool,
    ) {
        ui.visuals_mut().extreme_bg_color = Color32::from_gray(128);

        let viewport_rect = ui.available_rect_before_wrap();
        self.viewport_rect = viewport_rect;

        // ========================================================
        // 1. DYNAMIC TEXTURE ROUTING
        // ========================================================
        let mut egui_image = egui::Image::new(image_path).corner_radius(10.0);

        if let Some(tex) = dynamic_texture {
            // If the engine computed a new image, display that instead!
            egui_image = egui::Image::new(tex).corner_radius(10.0);

            let size = Vec2::new(tex.size()[0] as f32, tex.size()[1] as f32);
            if self.world_size != size {
                self.world_size = size;
                self.recenter_requested = true;
            }
        } else {
            // Fallback to loading from disk
            if let Ok(egui::load::TexturePoll::Ready { texture }) =
                egui_image.load_for_size(ui.ctx(), viewport_rect.size())
            {
                if self.world_size != texture.size {
                    self.world_size = texture.size;
                    self.recenter_requested = true;
                }
            }
        }

        // ========================================================
        // 2. DYNAMIC DIMENSIONS: Query the egui image cache!
        // ========================================================
        if let Ok(egui::load::TexturePoll::Ready { texture }) =
            egui_image.load_for_size(ui.ctx(), viewport_rect.size())
        {
            // If the loaded texture size differs from our current world_size, update it!
            if self.world_size != texture.size {
                self.world_size = texture.size;

                // Force a recenter because a new image just finished decoding
                self.recenter_requested = true;
            }
        }

        // 3. Handle Auto-Fit / Centering
        if self.recenter_requested {
            // Auto-fit math so giant images don't overflow the screen
            let scale_x = viewport_rect.width() / self.world_size.x;
            let scale_y = viewport_rect.height() / self.world_size.y;

            // Use .min(1.0) so small icons stay their true size, but massive photos shrink
            let fit_scale = scale_x.min(scale_y).min(1.0);

            self.transform.scaling = fit_scale;
            self.transform.translation = viewport_rect.center().to_vec2();
            self.recenter_requested = false;
        }

        let (id, rect) = ui.allocate_space(viewport_rect.size());
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        // Pan
        if response.dragged() {
            self.transform.translation += response.drag_delta();
        }

        // Zoom
        if response.hovered() {
            let zoom = ui.ctx().input(|i| i.zoom_delta());
            if zoom != 1.0 {
                if let Some(pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    let pos_in_scene = self.transform.inverse() * pos;
                    self.transform.scaling *= zoom;
                    self.transform.scaling = self.transform.scaling.clamp(0.05, 20.0);
                    self.transform.translation =
                        pos.to_vec2() - pos_in_scene.to_vec2() * self.transform.scaling;
                }
            }
        }

        // Draw Image (Centered coordinates)
        let world_rect = Rect::from_center_size(Pos2::ZERO, self.world_size);
        let screen_image_rect = self.transform.mul_rect(world_rect);

        ui.put(
            screen_image_rect,
            egui_image.fit_to_exact_size(screen_image_rect.size()), // We pass the image we constructed earlier!
        );

        // ==========================================
        // --- PERFECTLY ATTACHED HUD TABS ---
        // ==========================================
        let btn_size = Vec2::new(20.0, 60.0);
        let panel_bg = ui.ctx().style().visuals.panel_fill;

        // Left Tab
        let left_icon = if *left_panel_visible { "◀" } else { "▶" };
        egui::Area::new(egui::Id::new("left_toggle_area"))
            .fixed_pos(viewport_rect.left_center() + Vec2::new(-4.0, -btn_size.y / 2.0))
            .show(ui.ctx(), |ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;

                    let tab_radius = egui::CornerRadius {
                        nw: 0,
                        ne: 10,
                        sw: 0,
                        se: 10,
                    };
                    let btn = egui::Button::new(left_icon)
                        .corner_radius(tab_radius)
                        .fill(panel_bg);

                    if ui.add_sized(btn_size, btn).clicked() {
                        *left_panel_visible = !*left_panel_visible;
                    }
                });
            });

        // Right Tab
        let right_icon = if *right_panel_visible { "▶" } else { "◀" };
        egui::Area::new(egui::Id::new("right_toggle_area"))
            .fixed_pos(
                viewport_rect.right_center() + Vec2::new(-btn_size.x + 4.0, -btn_size.y / 2.0),
            )
            .show(ui.ctx(), |ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::NONE;

                    let tab_radius = egui::CornerRadius {
                        nw: 10,
                        ne: 0,
                        sw: 10,
                        se: 0,
                    };
                    let btn = egui::Button::new(right_icon)
                        .corner_radius(tab_radius)
                        .fill(panel_bg);

                    if ui.add_sized(btn_size, btn).clicked() {
                        *right_panel_visible = !*right_panel_visible;
                    }
                });
            });
    }
}
