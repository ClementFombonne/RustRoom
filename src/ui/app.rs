use eframe::*;
use egui::Vec2;
use egui_file_dialog::FileDialog;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use super::left_panel;
use super::right_panel::RightPanel;
use super::scene::ImageViewer;
use crate::database::{AlbumRecord, Catalog, ImageRecord};
use crate::engine::compute;

pub struct RustRoomApp {
    left_panel_visible: bool,
    right_panel_visible: bool,
    current_image: String,

    gallery: Vec<ImageRecord>,
    albums: Vec<AlbumRecord>,
    selected_album: Option<i64>,
    new_album_name: String,
    show_new_album_modal: bool,

    viewer: ImageViewer,
    right_panel: RightPanel,
    histogram_rx: Option<Receiver<Vec<f32>>>,

    // NEW: A receiver to know when the background import is done
    import_rx: Option<Receiver<bool>>,
    file_dialog: FileDialog,

    base_image: Option<Arc<image::DynamicImage>>,
    edited_texture: Option<egui::TextureHandle>,
    image_rx: Option<Receiver<Option<Arc<image::DynamicImage>>>>,
    edit_rx: Option<Receiver<egui::ColorImage>>,
    last_exposure: i32,
    needs_recompute: bool,
}

impl Default for RustRoomApp {
    fn default() -> Self {
        let mut app = Self {
            left_panel_visible: true,
            right_panel_visible: true,
            current_image: "".to_string(), // Starts empty
            gallery: Vec::new(),
            albums: Vec::new(),
            selected_album: None,
            new_album_name: String::new(),
            show_new_album_modal: false,
            viewer: ImageViewer::default(),
            right_panel: RightPanel::default(),
            histogram_rx: None,
            import_rx: None,
            file_dialog: FileDialog::new(),
            base_image: None,
            edited_texture: None,
            image_rx: None,
            edit_rx: None,
            last_exposure: 50,
            needs_recompute: false,
        };

        // Load existing images from the database on launch
        app.refresh_albums();
        app.refresh_gallery();

        // If we have images, select the first one automatically
        if let Some(first_img) = app.gallery.first() {
            app.current_image = first_img.original_path.clone();
            app.trigger_histogram_calculation(&app.current_image.clone());
        }

        app
    }
}

impl RustRoomApp {
    fn refresh_albums(&mut self) {
        if let Ok(catalog) = Catalog::new() {
            if let Ok(albums) = catalog.get_albums() {
                self.albums = albums;
            }
        }
    }

    fn refresh_gallery(&mut self) {
        if let Ok(catalog) = Catalog::new() {
            let images = match self.selected_album {
                Some(album_id) => catalog.get_images_for_album(album_id).unwrap_or_default(),
                None => catalog.get_all_images().unwrap_or_default(),
            };
            self.gallery = images;
        }
    }

    fn trigger_histogram_calculation(&mut self, image_path: &str) {
        let (tx, rx) = mpsc::channel();
        self.histogram_rx = Some(rx);
        let path_clone = image_path.to_string();

        thread::spawn(move || {
            if let Some(real_histogram) = compute::calculate_histogram(&path_clone) {
                let _ = tx.send(real_histogram);
            } else {
                let _ = tx.send(vec![0.0; 256]);
            }
        });
    }

    fn trigger_import(&mut self) {
        self.file_dialog.pick_multiple();
    }

    fn process_import(&mut self, file_paths: Vec<String>, target_album: Option<i64>) {
        if self.import_rx.is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.import_rx = Some(rx);

        thread::spawn(move || {
            match Catalog::new() {
                Ok(catalog) => {
                    let mut all_success = true;
                    for path in file_paths {
                        match catalog.import_photo(&path) {
                            Ok(image_id) => {
                                // Smart Feature: Automatically add imported photos to the active album
                                if let Some(album_id) = target_album {
                                    let _ = catalog.add_image_to_album(image_id, album_id);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to import {}: {}", path, e);
                                all_success = false;
                            }
                        }
                    }
                    let _ = tx.send(all_success);
                }
                Err(e) => {
                    eprintln!("[ERROR] Database locked: {}", e);
                    let _ = tx.send(false);
                }
            }
        });
    }

    fn trigger_edit(&mut self) {
        if let Some(img_arc) = &self.base_image {
            let (tx, rx) = mpsc::channel();
            self.edit_rx = Some(rx);

            // Clone the pointer, not the image! Blazing fast.
            let img_clone = Arc::clone(img_arc);
            let adj_clone = self.right_panel.adjustments.clone();

            thread::spawn(move || {
                let _ = tx.send(compute::apply_all_adjustments(&img_clone, &adj_clone));
            });
        }
    }
}

impl eframe::App for RustRoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let mut paths_to_import = None;
        {
            // We use a block scope so `state` releases the borrow lock on `self.file_dialog`!
            let state = self.file_dialog.update(ctx);
            if let Some(paths) = state.picked_multiple() {
                paths_to_import = Some(
                    paths
                        .into_iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect(),
                );
            } else if let Some(path) = state.picked() {
                paths_to_import = Some(vec![path.to_string_lossy().to_string()]);
            }
        }

        if let Some(file_paths) = paths_to_import {
            self.process_import(file_paths, self.selected_album);
            self.file_dialog = egui_file_dialog::FileDialog::new();
        }

        // --- CHECK BACKGROUND THREADS ---

        if let Some(rx) = &self.import_rx {
            match rx.try_recv() {
                Ok(success) => {
                    if success {
                        self.refresh_gallery();
                    }
                    self.import_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still importing, do nothing.
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[ERROR] Import thread disconnected unexpectedly.");
                    self.import_rx = None;
                }
            }
        }

        if let Some(rx) = &self.histogram_rx {
            match rx.try_recv() {
                Ok(new_histogram) => {
                    self.right_panel.histogram_data = new_histogram;
                    self.histogram_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still calculating, do nothing.
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[ERROR] Histogram thread disconnected unexpectedly.");
                    self.histogram_rx = None;
                }
            }
        }

        if let Some(rx) = &self.image_rx {
            if let Ok(img_opt) = rx.try_recv() {
                if let Some(img) = img_opt {
                    self.base_image = Some(img);
                    self.trigger_edit(); // Instantly apply default edits!
                }
                self.image_rx = None;
            }
        }

        if let Some(rx) = &self.edit_rx {
            if let Ok(color_image) = rx.try_recv() {
                // Convert raw pixels into an egui GPU texture
                self.edited_texture =
                    Some(ctx.load_texture("edited_img", color_image, egui::TextureOptions::LINEAR));
                self.edit_rx = None;

                // If the user kept dragging while we were calculating, run it again!
                if self.needs_recompute {
                    self.needs_recompute = false;
                    self.trigger_edit();
                }
            }
        }

        if self.right_panel.adjustments.exposure != self.last_exposure {
            self.last_exposure = self.right_panel.adjustments.exposure;

            // To prevent Thread Bombs, only spawn if one isn't already running
            if self.edit_rx.is_none() {
                self.trigger_edit();
            } else {
                self.needs_recompute = true;
            }
        }

        let image_path = self.current_image.clone();
        // --- TOP MENU BAR ---
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/RustRoom_icon.svg"))
                        .max_height(24.0),
                );
                ui.heading(egui::RichText::new("RustRoom").strong().size(14.0));
                ui.add_space(4.0);

                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("📥 Import Photos...").clicked() {
                            self.trigger_import();
                            ui.close();
                        }
                    });

                    // The Library Menu
                    ui.menu_button("Library", |ui| {
                        if ui
                            .button(if self.selected_album.is_none() {
                                "★ All Photos"
                            } else {
                                "🖼 All Photos"
                            })
                            .clicked()
                        {
                            self.selected_album = None;
                            self.refresh_gallery();
                            ui.close();
                        }

                        ui.separator();

                        let mut clicked_album = None;

                        // READ-ONLY LOCK STARTS
                        for album in &self.albums {
                            let is_selected = self.selected_album == Some(album.id);
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
                        // READ-ONLY LOCK ENDS

                        // Now we are safely outside the loop and can mutate `self`
                        if let Some(id) = clicked_album {
                            self.selected_album = Some(id);
                            self.refresh_gallery();
                        }

                        ui.separator();

                        // Create a new album natively
                        if ui.button("➕ New Album").clicked() {
                            self.show_new_album_modal = true;
                            ui.close();
                        }
                    });
                });
            });
        });

        if self.show_new_album_modal {
            egui::Window::new("Create New Album")
                .collapsible(false)
                .resizable(false)
                // Lock it to the dead center of the screen
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        // Auto-focus the text box so you can type immediately
                        let response = ui.add(egui::TextEdit::singleline(&mut self.new_album_name));
                        response.request_focus();
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_new_album_modal = false;
                            self.new_album_name.clear();
                        }

                        if ui.button("Create").clicked() && !self.new_album_name.is_empty() {
                            if let Ok(catalog) = Catalog::new() {
                                let _ = catalog.create_album(&self.new_album_name);
                                self.refresh_albums();
                                self.new_album_name.clear();
                            }
                            // Close the modal upon success
                            self.show_new_album_modal = false;
                        }
                    });
                });
        }

        // --- STATUS BAR ---
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!(
                        "📏 Dimensions: {:.0} x {:.0} px",
                        self.viewer.world_size.x, self.viewer.world_size.y
                    ));
                    ui.separator();
                    ui.label(format!(
                        "🔍 Zoom: {:.0}%",
                        self.viewer.transform.scaling * 100.0
                    ));
                    ui.separator();
                    ui.label(format!("📁 File: {}", self.current_image));

                    if self.histogram_rx.is_some() {
                        ui.separator();
                        ui.add(egui::Spinner::new().size(12.0));
                        ui.label("Computing Histogram...");
                    }

                    // NEW: Show a spinner while the image is importing
                    if self.import_rx.is_some() {
                        ui.separator();
                        ui.add(egui::Spinner::new().size(12.0));
                        ui.label("Importing & Generating Preview...");
                    }
                });
            });

        // --- BOTTOM PANEL (Image Gallery) ---
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                ui.heading("Image Gallery");
                ui.add_space(5.0);

                let mut clicked_image = None;

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Loop through our REAL database records!
                        for record in &self.gallery {
                            // Display the fast-loading preview generated during import
                            let thumb = egui::Image::new(format!("file://{}", record.preview_path))
                                .fit_to_exact_size(Vec2::new(120.0, 80.0))
                                .corner_radius(5);

                            if ui.add(egui::Button::image(thumb)).clicked() {
                                // But load the HIGH RES original into the central viewer!
                                clicked_image = Some(format!("file://{}", record.original_path));
                            }
                        }
                    });
                });

                if let Some(new_url) = clicked_image {
                    self.current_image = new_url.clone();
                    self.viewer.recenter_requested = true;

                    // Reset edit state for the new image
                    self.base_image = None;
                    self.edited_texture = None;
                    self.right_panel.adjustments.exposure = 50;
                    self.last_exposure = 50;

                    self.trigger_histogram_calculation(&new_url);

                    // Spawn a thread to load the image into memory
                    let (tx, rx) = mpsc::channel();
                    self.image_rx = Some(rx);
                    let path_clone = new_url.clone();
                    thread::spawn(move || {
                        let _ = tx.send(compute::load_image(&path_clone).map(Arc::new));
                    });
                }

                ui.add_space(5.0);
            });

        let clean_path = if image_path.starts_with("file://") {
            &image_path[7..]
        } else {
            &image_path
        };

        let current_record = self.gallery.iter().find(|r| r.original_path == clean_path);
        let metadata_json = current_record.map(|r| r.metadata_json.as_str());

        // --- LEFT PANEL, RIGHT PANEL, CENTRAL PANEL ---
        left_panel::show(
            ctx,
            self.left_panel_visible,
            &mut self.viewer,
            &image_path,
            metadata_json,
        );
        self.right_panel.show(ctx, self.right_panel_visible);
        self.viewer.show(
            ctx,
            &image_path,
            &self.edited_texture,
            &mut self.left_panel_visible,
            &mut self.right_panel_visible,
        );
    }
}
