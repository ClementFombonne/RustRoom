use eframe::*;
use egui::Vec2;
use egui_file_dialog::FileDialog;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use super::central_panel::scene::ImageViewer;
use super::left_panel;
use super::right_panel::RightPanel;
use crate::database::{AlbumRecord, Catalog, ImageRecord};
use crate::engine::compute;
use crate::ui::right_panel::PhotoAdjustments;

// FIX 1: Import our new dedicated worker system
use crate::engine::worker::{EditWorker, WorkerMessage};

pub struct RustRoomApp {
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub current_image: String,

    pub gallery: Vec<ImageRecord>,
    pub albums: Vec<AlbumRecord>,
    pub selected_album: Option<i64>,
    pub new_album_name: String,
    pub show_new_album_modal: bool,

    pub viewer: ImageViewer,
    pub right_panel: RightPanel,
    pub histogram_rx: Option<Receiver<Vec<f32>>>,
    pub import_rx: Option<Receiver<bool>>,
    pub file_dialog: FileDialog,

    pub edited_texture: Option<egui::TextureHandle>,
    pub image_rx: Option<Receiver<Option<Arc<image::DynamicImage>>>>,

    // FIX 2: Replace all the messy tracking variables with our clean Worker and a single state tracker!
    pub edit_worker: EditWorker,
    pub last_adjustments: PhotoAdjustments,
    pub is_computing_edit: bool,
}

impl Default for RustRoomApp {
    fn default() -> Self {
        let mut app = Self {
            left_panel_visible: true,
            right_panel_visible: true,
            current_image: "".to_string(),
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
            edited_texture: None,
            image_rx: None,

            // Initialize the dedicated background worker thread
            edit_worker: EditWorker::new(),
            last_adjustments: PhotoAdjustments::default(),
            is_computing_edit: false,
        };

        app.refresh_albums();
        app.refresh_gallery();

        if let Some(first_img) = app.gallery.first() {
            app.current_image = first_img.original_path.clone();
            app.trigger_histogram_calculation(&app.current_image.clone());
        }

        app
    }
}

impl RustRoomApp {
    pub fn refresh_albums(&mut self) {
        if let Ok(catalog) = Catalog::new() {
            if let Ok(albums) = catalog.get_albums() {
                self.albums = albums;
            }
        }
    }

    pub fn refresh_gallery(&mut self) {
        if let Ok(catalog) = Catalog::new() {
            let images = match self.selected_album {
                Some(album_id) => catalog.get_images_for_album(album_id).unwrap_or_default(),
                None => catalog.get_all_images().unwrap_or_default(),
            };
            self.gallery = images;
        }
    }

    pub fn trigger_histogram_calculation(&mut self, image_path: &str) {
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

    pub fn trigger_import(&mut self) {
        self.file_dialog.pick_multiple();
    }

    pub fn process_import(&mut self, file_paths: Vec<String>, target_album: Option<i64>) {
        if self.import_rx.is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.import_rx = Some(rx);

        thread::spawn(move || match Catalog::new() {
            Ok(catalog) => {
                let mut all_success = true;
                for path in file_paths {
                    match catalog.import_photo(&path) {
                        Ok(image_id) => {
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
        });
    }

    // FIX 3: `trigger_edit` has been entirely DELETED. The worker handles it now!
}

impl eframe::App for RustRoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let mut paths_to_import = None;
        {
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

        if let Some(rx) = &self.image_rx {
            if let Ok(img_opt) = rx.try_recv() {
                if let Some(img) = img_opt {
                    let _ = self.edit_worker.tx.send(WorkerMessage::LoadImage(img));
                    // Pass ctx.clone() and set the computing flag
                    let _ = self.edit_worker.tx.send(WorkerMessage::Adjust(
                        self.last_adjustments.clone(),
                        ctx.clone(),
                    ));
                    self.is_computing_edit = true;
                }
                self.image_rx = None;
            }
        }

        // --- THE NEW RENDER PIPELINE ---

        // 1. Did the worker finish a new frame?
        if let Ok((color_image, new_histogram)) = self.edit_worker.result_rx.try_recv() {
            self.edited_texture =
                Some(ctx.load_texture("edited_img", color_image, egui::TextureOptions::LINEAR));

            // Update the live histogram
            self.right_panel.histogram_data = new_histogram;
            self.is_computing_edit = false;
        }

        // 2. Did the user move ANY slider?
        if self.right_panel.adjustments != self.last_adjustments {
            self.last_adjustments = self.right_panel.adjustments.clone();

            // Pass ctx.clone() to the worker
            let _ = self.edit_worker.tx.send(WorkerMessage::Adjust(
                self.last_adjustments.clone(),
                ctx.clone(),
            ));
            self.is_computing_edit = true; // STARTED COMPUTING!
        }

        if let Some(rx) = &self.histogram_rx {
            match rx.try_recv() {
                Ok(new_histogram) => {
                    self.right_panel.histogram_data = new_histogram;
                    self.histogram_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.histogram_rx = None;
                }
            }
        }

        // --- IMAGE LOADING ---
        if let Some(rx) = &self.image_rx {
            if let Ok(img_opt) = rx.try_recv() {
                if let Some(img) = img_opt {
                    // Send the new base image to the persistent worker thread
                    let _ = self.edit_worker.tx.send(WorkerMessage::LoadImage(img));
                    // Instantly trigger an adjustment with the current defaults
                    let _ = self.edit_worker.tx.send(WorkerMessage::Adjust(
                        self.last_adjustments.clone(),
                        ctx.clone(),
                    ));
                }
                self.image_rx = None;
            }
        }

        let image_path = self.current_image.clone();

        // --- TOP MENU BAR ---
        super::menu_bar::show(ctx, self);

        if self.show_new_album_modal {
            egui::Window::new("Create New Album")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
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
                            self.show_new_album_modal = false;
                        }
                    });
                });
        }

        // --- STATUS BAR ---
        super::status_bar::show(ctx, self);

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
                        for record in &self.gallery {
                            let thumb = egui::Image::new(format!("file://{}", record.preview_path))
                                .fit_to_exact_size(Vec2::new(120.0, 80.0))
                                .corner_radius(5);

                            if ui.add(egui::Button::image(thumb)).clicked() {
                                clicked_image = Some(format!("file://{}", record.original_path));
                            }
                        }
                    });
                });

                if let Some(new_url) = clicked_image {
                    self.current_image = new_url.clone();
                    self.viewer.recenter_requested = true;

                    self.edited_texture = None;

                    // Restore default state
                    let defaults = PhotoAdjustments::default();
                    self.right_panel.adjustments = defaults.clone();
                    self.last_adjustments = defaults;

                    self.trigger_histogram_calculation(&new_url);

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
        super::central_panel::show(
            ctx,
            &mut self.viewer,
            &image_path,
            &self.edited_texture,
            &mut self.left_panel_visible,
            &mut self.right_panel_visible,
        );
    }
}
