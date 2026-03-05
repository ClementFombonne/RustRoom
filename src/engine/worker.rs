use super::compute;
use crate::ui::right_panel::PhotoAdjustments;
use image::DynamicImage;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub enum WorkerMessage {
    LoadImage(Arc<DynamicImage>),
    Adjust(PhotoAdjustments, egui::Context),
}

pub struct EditWorker {
    pub tx: Sender<WorkerMessage>,
    pub result_rx: Receiver<(egui::ColorImage, Vec<f32>)>, // Color, Histogram
}

impl Default for EditWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl EditWorker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<WorkerMessage>();
        let (result_tx, result_rx) = mpsc::channel::<(egui::ColorImage, Vec<f32>)>();

        thread::spawn(move || {
            let mut current_image: Option<Arc<DynamicImage>> = None;

            loop {
                // Block until the UI sends us something
                match rx.recv() {
                    Ok(first_msg) => {
                        let mut latest_adj = None;

                        // Process the first message
                        match first_msg {
                            WorkerMessage::LoadImage(img) => current_image = Some(img),
                            WorkerMessage::Adjust(adj, ctx) => latest_adj = Some((adj, ctx)),
                        }

                        while let Ok(newer_msg) = rx.try_recv() {
                            match newer_msg {
                                WorkerMessage::LoadImage(img) => current_image = Some(img),
                                WorkerMessage::Adjust(adj, ctx) => latest_adj = Some((adj, ctx)),
                            }
                        }

                        // Finally, if we ended up with an adjustment request, run it!
                        if let Some((adj, ctx)) = latest_adj {
                            if let Some(img) = &current_image {
                                // let mut rgb_img = img.to_rgb8();

                                let color_image = compute::apply_all_adjustments(img, &adj);
                                let new_histogram =
                                    compute::calculate_histogram_from_buffer(color_image.as_raw());
                                let _ = result_tx.send((color_image, new_histogram));

                                ctx.request_repaint();
                            }
                        }
                    }
                    Err(_) => break, // App closed
                }
            }
        });

        Self { tx, result_rx }
    }
}
