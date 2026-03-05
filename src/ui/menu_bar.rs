use crate::ui::app::RustRoomApp;
use egui::Context;

pub fn show(ctx: &Context, app: &mut RustRoomApp) {
    egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../assets/RustRoom_icon.svg"))
                    .max_height(24.0),
            );
            ui.heading(egui::RichText::new("RustRoom").strong().size(16.0));
            ui.add_space(4.0);

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📥 Import Photos...").clicked() {
                        app.trigger_import();
                        ui.close();
                    }
                });

                ui.menu_button("Library", |ui| {
                    if ui
                        .button(if app.selected_album.is_none() {
                            "★ All Photos"
                        } else {
                            "🖼 All Photos"
                        })
                        .clicked()
                    {
                        app.selected_album = None;
                        app.refresh_gallery();
                        ui.close();
                    }

                    ui.separator();

                    let mut clicked_album = None;
                    for album in &app.albums {
                        let is_selected = app.selected_album == Some(album.id);
                        let label = if is_selected {
                            format!("★ {}", album.name)
                        } else {
                            album.name.clone()
                        };
                        if ui.button(label).clicked() {
                            clicked_album = Some(album.id);
                            ui.close();
                        }
                    }

                    if let Some(id) = clicked_album {
                        app.selected_album = Some(id);
                        app.refresh_gallery();
                    }

                    ui.separator();

                    if ui.button("➕ New Album").clicked() {
                        app.show_new_album_modal = true;
                        ui.close();
                    }
                });
            });
        });
    });
}
